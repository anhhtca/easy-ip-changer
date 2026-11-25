use std::ffi::OsStr;
use std::net::Ipv4Addr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use windows::Win32::Foundation::{CloseHandle, ERROR_BUFFER_OVERFLOW, ERROR_CANCELLED, HANDLE, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, GetIfEntry2, IF_TYPE_ETHERNET_CSMACD,
    IF_TYPE_IEEE80211, IP_ADAPTER_ADDRESSES_LH, MIB_IF_ROW2,
};
use windows::Win32::NetworkManagement::Ndis::{
    IfOperStatusUp, NdisPhysicalMedium802_3, NdisPhysicalMediumBluetooth,
    NdisPhysicalMediumNative802_11, NdisPhysicalMediumUnspecified, NdisPhysicalMediumWirelessLan,
};
use windows::Win32::Networking::WinSock::{AF_INET, SOCKADDR_IN};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use crate::domain::{
    ApplyReport, IpConfigRequest, NetworkAdapter, NicKind, build_apply_report,
    build_netsh_commands, is_usb_ethernet_hint, is_virtual_adapter_name, sort_and_pick_default_nic,
};
use crate::platform::SystemNetworkBackend;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct WindowsBackend;

impl SystemNetworkBackend for WindowsBackend {
    fn is_elevated(&self) -> bool {
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut return_len: u32 = 0;
            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_len,
            );
            let _ = CloseHandle(token);

            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    fn relaunch_elevated(&self) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Không xác định được đường dẫn ứng dụng: {e}"))?;
        let wide_path: Vec<u16> = OsStr::new(exe_path.as_os_str())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = unsafe {
            ShellExecuteW(
                None,
                w!("runas"),
                PCWSTR(wide_path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };

        let code = result.0 as isize;
        if code > 32 {
            std::process::exit(0);
        }

        if code == ERROR_CANCELLED.0 as isize || code == 5 {
            Err(
                "Bạn đã hủy hộp thoại cấp quyền. Vui lòng bấm lại nút trên và chọn 'Yes' để tiếp tục."
                    .to_string(),
            )
        } else {
            Err(format!(
                "Không thể yêu cầu quyền Quản trị (mã lỗi {code}). Vui lòng nhấp chuột phải vào file ứng dụng và chọn 'Run as administrator'."
            ))
        }
    }

    fn list_physical_adapters(&self) -> Result<Vec<NetworkAdapter>, String> {
        let mut buf_len: u32 = 32 * 1024;
        let mut buffer = vec![0u8; buf_len as usize];

        let status = unsafe {
            let first = GetAdaptersAddresses(
                AF_INET.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                Some(buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>()),
                &mut buf_len,
            );
            if first == ERROR_BUFFER_OVERFLOW.0 {
                buffer.resize(buf_len as usize, 0);
                GetAdaptersAddresses(
                    AF_INET.0 as u32,
                    GAA_FLAG_INCLUDE_PREFIX,
                    None,
                    Some(buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>()),
                    &mut buf_len,
                )
            } else {
                first
            }
        };

        if status != NO_ERROR.0 {
            return Err(format!(
                "Không thể đọc danh sách card mạng từ Windows (mã lỗi {status})."
            ));
        }

        let mut adapters = Vec::new();
        let mut ptr = buffer.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();

        while !ptr.is_null() {
            let adapter = unsafe { &*ptr };
            let if_type = adapter.IfType;
            let kind = match if_type {
                IF_TYPE_ETHERNET_CSMACD => Some(NicKind::Ethernet),
                IF_TYPE_IEEE80211 => Some(NicKind::Wifi),
                _ => None,
            };

            if let Some(kind) = kind {
                let friendly_name = unsafe { pwstr_to_string(adapter.FriendlyName) };
                let description = unsafe { pwstr_to_string(adapter.Description) };

                if !is_virtual_adapter_name(&friendly_name, &description) {
                    let mut row = MIB_IF_ROW2 {
                        InterfaceLuid: adapter.Luid,
                        ..Default::default()
                    };
                    let row_ok = unsafe { GetIfEntry2(&mut row) == NO_ERROR };

                    if row_ok {
                        // Bit 0 = HardwareInterface, Bit 2 = ConnectorPresent trong MIB_IF_ROW2
                        let flags = row.InterfaceAndOperStatusFlags._bitfield;
                        let is_hardware = (flags & 0b0001) != 0;
                        let connector_present = (flags & 0b0100) != 0;
                        let medium = row.PhysicalMediumType;

                        let valid_medium = match kind {
                            NicKind::Ethernet => {
                                medium == NdisPhysicalMedium802_3
                                    || medium == NdisPhysicalMediumUnspecified
                            }
                            NicKind::Wifi => {
                                medium == NdisPhysicalMediumNative802_11
                                    || medium == NdisPhysicalMediumWirelessLan
                                    || medium == NdisPhysicalMediumUnspecified
                            }
                        };

                        if is_hardware
                            && connector_present
                            && medium != NdisPhysicalMediumBluetooth
                            && valid_medium
                        {
                            let if_index = unsafe { adapter.Anonymous1.Anonymous.IfIndex };
                            if if_index > 0 {
                                let current_ipv4 = unsafe { extract_first_ipv4(adapter) };
                                let is_usb = kind == NicKind::Ethernet
                                    && is_usb_ethernet_hint(&friendly_name, &description);

                                adapters.push(NetworkAdapter {
                                    if_index,
                                    friendly_name,
                                    description,
                                    kind,
                                    is_usb_ethernet: is_usb,
                                    is_connected: adapter.OperStatus == IfOperStatusUp,
                                    current_ipv4,
                                });
                            }
                        }
                    }
                }
            }

            ptr = adapter.Next;
        }

        sort_and_pick_default_nic(&mut adapters);
        Ok(adapters)
    }

    fn apply_config(&self, req: &IpConfigRequest) -> Result<ApplyReport, String> {
        let netsh_path = resolve_netsh_path();
        let commands = build_netsh_commands(req);

        for args in commands {
            let output = Command::new(&netsh_path)
                .args(&args)
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Không thể chạy công cụ cấu hình mạng Windows (netsh): {e}"))?;

            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let detail = if !stderr.is_empty() { stderr } else { stdout };
                return Err(translate_netsh_error(&detail));
            }
        }

        Ok(build_apply_report(req))
    }

    fn is_preview_mode(&self) -> bool {
        false
    }
}

fn resolve_netsh_path() -> PathBuf {
    let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    PathBuf::from(sys_root).join("System32").join("netsh.exe")
}

fn translate_netsh_error(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("elevation") || lower.contains("access is denied") || lower.contains("privilege") {
        "Ứng dụng chưa có đủ quyền Quản trị (Administrator). Vui lòng mở lại bằng quyền Quản trị."
            .to_string()
    } else if lower.contains("already") && lower.contains("exists") {
        "Địa chỉ IP này đang bị trùng hoặc đã được gán cho một kết nối khác trên máy.".to_string()
    } else if lower.contains("element not found") || lower.contains("invalid interface") {
        "Không tìm thấy card mạng đã chọn (có thể cáp USB mạng vừa bị rút ra). Hãy quay lại và bấm Quét lại."
            .to_string()
    } else if lower.contains("media is disconnected") {
        "Cáp mạng đang chưa cắm hoặc card mạng đang tắt. Hãy kiểm tra lại kết nối rồi thử lại."
            .to_string()
    } else if raw.is_empty() {
        "Windows từ chối áp dụng cấu hình mạng. Vui lòng kiểm tra lại thông số và thử lại."
            .to_string()
    } else {
        format!("Không thể lưu cấu hình mạng: {raw}")
    }
}

unsafe fn pwstr_to_string(ptr: windows::core::PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { ptr.to_string().unwrap_or_default() }
}

unsafe fn extract_first_ipv4(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
    let mut unicast = adapter.FirstUnicastAddress;
    while !unicast.is_null() {
        let addr_entry = unsafe { &*unicast };
        let sockaddr = addr_entry.Address.lpSockaddr;
        if !sockaddr.is_null() {
            let family = unsafe { (*sockaddr).sa_family };
            if family == AF_INET {
                let ipv4 = unsafe { &*(sockaddr.cast::<SOCKADDR_IN>()) };
                let bytes = unsafe { ipv4.sin_addr.S_un.S_un_b };
                let ip = Ipv4Addr::new(bytes.s_b1, bytes.s_b2, bytes.s_b3, bytes.s_b4);
                // Bỏ qua địa chỉ APIPA 169.254.x.x nếu còn địa chỉ khác, hoặc trả về cuối cùng
                if ip.octets()[0] != 169 || ip.octets()[1] != 254 {
                    return Some(ip.to_string());
                }
            }
        }
        unicast = unsafe { (*unicast).Next };
    }
    None
}
