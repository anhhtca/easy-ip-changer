use std::net::Ipv4Addr;
use thiserror::Error;

pub const DEFAULT_SUBNET: &str = "255.255.255.0";
pub const DEFAULT_DNS_PRIMARY: &str = "10.0.58.11";
pub const DEFAULT_DNS_SECONDARY: &str = "10.0.29.5";

#[cfg_attr(not(windows), allow(dead_code))]
pub const METRIC_HIGH_PRIORITY: u32 = 10;
#[cfg_attr(not(windows), allow(dead_code))]
pub const METRIC_LOW_PRIORITY: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NicKind {
    Ethernet,
    Wifi,
}

impl NicKind {
    pub fn display_label(self, is_usb: bool) -> &'static str {
        match (self, is_usb) {
            (NicKind::Ethernet, true) => "Ethernet (USB)",
            (NicKind::Ethernet, false) => "Ethernet",
            (NicKind::Wifi, _) => "Wi-Fi",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapter {
    pub if_index: u32,
    pub friendly_name: String,
    pub description: String,
    pub kind: NicKind,
    pub is_usb_ethernet: bool,
    pub is_connected: bool,
    pub current_ipv4: Option<String>,
}

impl NetworkAdapter {
    pub fn status_text(&self) -> &'static str {
        match (self.kind, self.is_connected) {
            (_, true) => "Đang kết nối",
            (NicKind::Ethernet, false) => "Chưa cắm dây mạng",
            (NicKind::Wifi, false) => "Chưa kết nối Wi-Fi",
        }
    }

    pub fn ip_summary(&self) -> String {
        match &self.current_ipv4 {
            Some(ip) if !ip.is_empty() => format!("IP hiện tại: {ip}"),
            _ => "Chưa có địa chỉ IP".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PriorityMode {
    /// Wi-Fi ưu tiên cao hơn Ethernet (metric 10 vs 50)
    #[default]
    PreferWifi,
    /// Ethernet ưu tiên cao hơn Wi-Fi (metric 10 vs 50)
    PreferEthernet,
    /// Để Windows tự quyết định (automatic metric)
    WindowsDefault,
}

impl PriorityMode {
    pub fn from_index(idx: i32) -> Self {
        match idx {
            1 => Self::PreferEthernet,
            2 => Self::WindowsDefault,
            _ => Self::PreferWifi,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityUpdate {
    pub eth_if_index: u32,
    pub eth_name: String,
    pub wifi_if_index: u32,
    pub wifi_name: String,
    pub mode: PriorityMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpConfigRequest {
    pub adapter_if_index: u32,
    pub adapter_name: String,
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub gateway: Option<Ipv4Addr>,
    pub dns_primary: Option<Ipv4Addr>,
    pub dns_secondary: Option<Ipv4Addr>,
    pub priority: Option<PriorityUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReport {
    pub adapter_name: String,
    pub ip: String,
    pub subnet: String,
    pub gateway: String,
    pub dns_primary: String,
    pub dns_secondary: String,
    pub priority_summary: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Vui lòng nhập đầy đủ 4 nhóm số cho địa chỉ IP (mỗi nhóm từ 0 đến 255).")]
    IncompleteIp,
    #[error("Nhóm số thứ {index} của địa chỉ IP không hợp lệ (chỉ nhập từ 0 đến 255).")]
    InvalidIpOctet { index: usize },
    #[error("Địa chỉ IP này thuộc dải hệ thống không dùng được ({reason}). Vui lòng kiểm tra lại.")]
    ReservedIp { reason: &'static str },
    #[error("Địa chỉ IP không thể là địa chỉ mạng hoặc broadcast của Subnet Mask hiện tại.")]
    NetworkOrBroadcastIp,
    #[error("Subnet Mask không hợp lệ. Ví dụ đúng: 255.255.255.0")]
    InvalidSubnetMask,
    #[error("{field} không đúng định dạng địa chỉ IPv4 (ví dụ: 10.0.58.1).")]
    InvalidOptionalIpv4 { field: &'static str },
    #[error("Default Gateway không được trùng với Địa chỉ IP của máy.")]
    GatewayEqualsIp,
}

/// Lọc chỉ giữ lại chữ số (tối đa 3 ký tự) cho 1 ô octet IP.
pub fn sanitize_octet_digits(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_digit())
        .take(3)
        .collect()
}

/// Nếu người dùng dán nguyên địa chỉ IPv4 (vd "10.0.58.120"), tách thành 4 chuỗi octet.
pub fn parse_pasted_ipv4(raw: &str) -> Option<[String; 4]> {
    let trimmed = raw.trim();
    if !trimmed.contains('.') {
        return None;
    }
    let parts: Vec<&str> = trimmed.split('.').map(str::trim).collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out = [String::new(), String::new(), String::new(), String::new()];
    for (i, part) in parts.iter().enumerate() {
        let val: u8 = part.parse().ok()?;
        out[i] = val.to_string();
    }
    Some(out)
}

/// Tự động sinh Default Gateway từ 3 nhóm đầu của IP + ".1".
pub fn derive_gateway(o1: &str, o2: &str, o3: &str) -> String {
    let Ok(a) = o1.trim().parse::<u8>() else {
        return String::new();
    };
    let Ok(b) = o2.trim().parse::<u8>() else {
        return String::new();
    };
    let Ok(c) = o3.trim().parse::<u8>() else {
        return String::new();
    };
    format!("{a}.{b}.{c}.1")
}

pub fn validate_ip_octets(octets: [&str; 4]) -> Result<Ipv4Addr, ValidationError> {
    let mut nums = [0u8; 4];
    for (i, raw) in octets.iter().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::IncompleteIp);
        }
        nums[i] = trimmed
            .parse::<u8>()
            .map_err(|_| ValidationError::InvalidIpOctet { index: i + 1 })?;
    }

    let addr = Ipv4Addr::from(nums);
    if nums[0] == 0 {
        return Err(ValidationError::ReservedIp {
            reason: "nhóm đầu tiên không thể là 0",
        });
    }
    if nums[0] == 127 {
        return Err(ValidationError::ReservedIp {
            reason: "dải 127.x.x.x là địa chỉ nội bộ Loopback",
        });
    }
    if nums[0] >= 224 {
        return Err(ValidationError::ReservedIp {
            reason: "nhóm đầu từ 224 trở lên thuộc dải Multicast/dự phòng",
        });
    }
    if addr == Ipv4Addr::BROADCAST {
        return Err(ValidationError::ReservedIp {
            reason: "địa chỉ Broadcast toàn mạng",
        });
    }
    Ok(addr)
}

pub fn validate_subnet_mask(raw: &str) -> Result<Ipv4Addr, ValidationError> {
    let addr: Ipv4Addr = raw
        .trim()
        .parse()
        .map_err(|_| ValidationError::InvalidSubnetMask)?;
    let bits = u32::from(addr);
    if bits == 0 || bits == u32::MAX {
        return Err(ValidationError::InvalidSubnetMask);
    }
    let inv = !bits;
    if inv.checked_add(1).is_some_and(u32::is_power_of_two) {
        Ok(addr)
    } else {
        Err(ValidationError::InvalidSubnetMask)
    }
}

pub fn validate_optional_ipv4(
    raw: &str,
    field: &'static str,
) -> Result<Option<Ipv4Addr>, ValidationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let addr: Ipv4Addr = trimmed
        .parse()
        .map_err(|_| ValidationError::InvalidOptionalIpv4 { field })?;
    if addr == Ipv4Addr::UNSPECIFIED || addr == Ipv4Addr::BROADCAST {
        return Err(ValidationError::InvalidOptionalIpv4 { field });
    }
    Ok(Some(addr))
}

pub fn validate_ip_consistency(
    ip: Ipv4Addr,
    subnet: Ipv4Addr,
    gateway: Option<Ipv4Addr>,
) -> Result<(), ValidationError> {
    let ip_u32 = u32::from(ip);
    let mask_u32 = u32::from(subnet);
    let host_mask = !mask_u32;

    // Với subnet có từ 4 địa chỉ trở lên (/1 đến /30), không cho phép địa chỉ network hay broadcast.
    if host_mask >= 3 {
        let host_part = ip_u32 & host_mask;
        if host_part == 0 || host_part == host_mask {
            return Err(ValidationError::NetworkOrBroadcastIp);
        }
    }

    if let Some(gw) = gateway
        && gw == ip
    {
        return Err(ValidationError::GatewayEqualsIp);
    }

    Ok(())
}

/// Kiểm tra tên/mô tả có phải adapter ảo/VPN/Bluetooth không.
/// Luôn giữ lại Ethernet vật lý, USB Ethernet và Wi-Fi vật lý.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn is_virtual_adapter_name(friendly_name: &str, description: &str) -> bool {
    let combined = format!(
        "{} {}",
        friendly_name.to_ascii_lowercase(),
        description.to_ascii_lowercase()
    );
    const VIRTUAL_KEYWORDS: &[&str] = &[
        "virtual",
        "vmware",
        "virtualbox",
        "vbox",
        "hyper-v",
        "vethernet",
        "bluetooth",
        "wi-fi direct",
        "wifi direct",
        "wan miniport",
        "tap-windows",
        "tap-nord",
        "wintun",
        "wireguard",
        "zerotier",
        "tailscale",
        "openvpn",
        "cisco anyconnect",
        "fortinet",
        "juniper",
        "palo alto",
        "globalprotect",
        "radmin vpn",
        "hamachi",
        "npcap",
        "loopback",
        "kernel debug",
        "teredo",
        "isatap",
        "6to4",
        "ras async",
        "wsl",
        "docker",
    ];
    VIRTUAL_KEYWORDS.iter().any(|kw| combined.contains(kw))
}

/// Nhận diện card USB Ethernet để hiển thị nhãn thân thiện cho người dùng.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn is_usb_ethernet_hint(friendly_name: &str, description: &str) -> bool {
    let combined = format!(
        "{} {}",
        friendly_name.to_ascii_lowercase(),
        description.to_ascii_lowercase()
    );
    const USB_HINTS: &[&str] = &[
        "usb",
        "asix",
        "ax88179",
        "ax88772",
        "rtl8153",
        "rtl8156",
        "displaylink",
        "type-c",
        "thunderbolt",
        "dock",
        "ue300",
        "tp-link",
    ];
    USB_HINTS.iter().any(|kw| combined.contains(kw))
}

/// Kiểm tra danh sách có chứa ít nhất một card Ethernet (bao gồm USB Ethernet) hay không.
pub fn has_ethernet_adapter(adapters: &[NetworkAdapter]) -> bool {
    adapters.iter().any(|a| a.kind == NicKind::Ethernet)
}

/// Sắp xếp card mạng: Ethernet đang kết nối -> Ethernet chưa cắm dây -> Wi-Fi đang kết nối -> Wi-Fi chưa kết nối.
/// Chỉ tự động chọn mặc định khi có ít nhất 1 card Ethernet (không bao giờ tự động chọn Wi-Fi).
pub fn sort_and_pick_default_nic(adapters: &mut [NetworkAdapter]) -> Option<usize> {
    adapters.sort_by(|a, b| {
        let rank = |nic: &NetworkAdapter| -> u8 {
            match (nic.kind, nic.is_connected) {
                (NicKind::Ethernet, true) => 0,
                (NicKind::Ethernet, false) => 1,
                (NicKind::Wifi, true) => 2,
                (NicKind::Wifi, false) => 3,
            }
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.friendly_name.to_lowercase().cmp(&b.friendly_name.to_lowercase()))
    });

    adapters
        .first()
        .filter(|first| first.kind == NicKind::Ethernet)
        .map(|_| 0)
}

/// Tìm cặp (Ethernet, Wi-Fi) để áp dụng ưu tiên dựa trên card người dùng đang chọn.
pub fn resolve_priority_pair(
    adapters: &[NetworkAdapter],
    selected_index: usize,
) -> Option<(&NetworkAdapter, &NetworkAdapter)> {
    let selected = adapters.get(selected_index)?;
    match selected.kind {
        NicKind::Ethernet => {
            let wifi = adapters
                .iter()
                .find(|a| a.kind == NicKind::Wifi && a.is_connected)
                .or_else(|| adapters.iter().find(|a| a.kind == NicKind::Wifi))?;
            Some((selected, wifi))
        }
        NicKind::Wifi => {
            let eth = adapters
                .iter()
                .find(|a| a.kind == NicKind::Ethernet && a.is_connected)
                .or_else(|| adapters.iter().find(|a| a.kind == NicKind::Ethernet))?;
            Some((eth, selected))
        }
    }
}

/// Sinh danh sách tham số lệnh `netsh.exe` để áp dụng IP, Subnet, Gateway, DNS và Metric.
/// Sử dụng số hiệu `if_index` thay cho tên chữ để tránh mọi lỗi mã hóa Unicode trên Windows.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn build_netsh_commands(req: &IpConfigRequest) -> Vec<Vec<String>> {
    let mut cmds = Vec::new();
    let idx = req.adapter_if_index;

    // 1. Static IPv4 + Subnet Mask + Default Gateway
    let mut ip_cmd = vec![
        "interface".into(),
        "ipv4".into(),
        "set".into(),
        "address".into(),
        format!("name={idx}"),
        "source=static".into(),
        format!("address={}", req.ip),
        format!("mask={}", req.subnet),
    ];
    match req.gateway {
        Some(gw) => ip_cmd.push(format!("gateway={gw}")),
        None => ip_cmd.push("gateway=none".into()),
    }
    cmds.push(ip_cmd);

    // 2. DNS Servers (validate=no giúp ghi ngay lập tức không chờ truy vấn DNS)
    match (req.dns_primary, req.dns_secondary) {
        (Some(d1), Some(d2)) => {
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "set".into(),
                "dnsservers".into(),
                format!("name={idx}"),
                "source=static".into(),
                format!("address={d1}"),
                "validate=no".into(),
            ]);
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "add".into(),
                "dnsservers".into(),
                format!("name={idx}"),
                format!("address={d2}"),
                "index=2".into(),
                "validate=no".into(),
            ]);
        }
        (Some(d1), None) | (None, Some(d1)) => {
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "set".into(),
                "dnsservers".into(),
                format!("name={idx}"),
                "source=static".into(),
                format!("address={d1}"),
                "validate=no".into(),
            ]);
        }
        (None, None) => {
            cmds.push(vec![
                "interface".into(),
                "ipv4".into(),
                "set".into(),
                "dnsservers".into(),
                format!("name={idx}"),
                "source=static".into(),
                "address=none".into(),
            ]);
        }
    }

    // 3. Interface Metric giữa Ethernet và Wi-Fi (nếu người dùng bật checkbox)
    if let Some(prio) = &req.priority {
        match prio.mode {
            PriorityMode::PreferWifi => {
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.wifi_if_index.to_string(),
                    format!("metric={METRIC_HIGH_PRIORITY}"),
                ]);
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.eth_if_index.to_string(),
                    format!("metric={METRIC_LOW_PRIORITY}"),
                ]);
            }
            PriorityMode::PreferEthernet => {
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.eth_if_index.to_string(),
                    format!("metric={METRIC_HIGH_PRIORITY}"),
                ]);
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.wifi_if_index.to_string(),
                    format!("metric={METRIC_LOW_PRIORITY}"),
                ]);
            }
            PriorityMode::WindowsDefault => {
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.eth_if_index.to_string(),
                    "metric=automatic".into(),
                ]);
                cmds.push(vec![
                    "interface".into(),
                    "ipv4".into(),
                    "set".into(),
                    "interface".into(),
                    prio.wifi_if_index.to_string(),
                    "metric=automatic".into(),
                ]);
            }
        }
    }

    cmds
}

pub fn build_apply_report(req: &IpConfigRequest) -> ApplyReport {
    let priority_summary = req.priority.as_ref().map(|p| match p.mode {
        PriorityMode::PreferWifi => format!("Ưu tiên {} hơn {}", p.wifi_name, p.eth_name),
        PriorityMode::PreferEthernet => format!("Ưu tiên {} hơn {}", p.eth_name, p.wifi_name),
        PriorityMode::WindowsDefault => {
            format!("Mặc định Windows ({} & {})", p.eth_name, p.wifi_name)
        }
    });

    ApplyReport {
        adapter_name: req.adapter_name.clone(),
        ip: req.ip.to_string(),
        subnet: req.subnet.to_string(),
        gateway: req
            .gateway
            .map_or_else(|| "Không thiết lập".to_string(), |g| g.to_string()),
        dns_primary: req
            .dns_primary
            .map_or_else(|| "Không thiết lập".to_string(), |d| d.to_string()),
        dns_secondary: req
            .dns_secondary
            .map_or_else(|| "Không thiết lập".to_string(), |d| d.to_string()),
        priority_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_gateway() {
        assert_eq!(derive_gateway("10", "0", "58"), "10.0.58.1");
        assert_eq!(derive_gateway("192", "168", "10"), "192.168.10.1");
        assert_eq!(derive_gateway("10", "", "58"), "");
        assert_eq!(derive_gateway("256", "0", "1"), "");
    }

    #[test]
    fn test_parse_pasted_ipv4() {
        assert_eq!(
            parse_pasted_ipv4(" 10.0.58.120 "),
            Some(["10".into(), "0".into(), "58".into(), "120".into()])
        );
        assert_eq!(parse_pasted_ipv4("192.168.1.999"), None);
        assert_eq!(parse_pasted_ipv4("192"), None);
    }

    #[test]
    fn test_validate_ip_octets() {
        assert_eq!(
            validate_ip_octets(["10", "0", "58", "120"]).unwrap(),
            Ipv4Addr::new(10, 0, 58, 120)
        );
        assert!(matches!(
            validate_ip_octets(["10", "0", "", "1"]),
            Err(ValidationError::IncompleteIp)
        ));
        assert!(matches!(
            validate_ip_octets(["10", "300", "1", "1"]),
            Err(ValidationError::InvalidIpOctet { index: 2 })
        ));
        assert!(matches!(
            validate_ip_octets(["127", "0", "0", "1"]),
            Err(ValidationError::ReservedIp { .. })
        ));
        assert!(matches!(
            validate_ip_octets(["0", "0", "0", "0"]),
            Err(ValidationError::ReservedIp { .. })
        ));
        assert!(matches!(
            validate_ip_octets(["224", "0", "0", "1"]),
            Err(ValidationError::ReservedIp { .. })
        ));
    }

    #[test]
    fn test_validate_subnet_mask() {
        assert!(validate_subnet_mask("255.255.255.0").is_ok());
        assert!(validate_subnet_mask("255.255.254.0").is_ok());
        assert!(validate_subnet_mask("255.255.0.0").is_ok());
        assert!(validate_subnet_mask("255.0.255.0").is_err());
        assert!(validate_subnet_mask("0.0.0.0").is_err());
        assert!(validate_subnet_mask("255.255.255.255").is_err());
    }

    #[test]
    fn test_validate_ip_consistency() {
        let mask = Ipv4Addr::new(255, 255, 255, 0);
        assert!(validate_ip_consistency(Ipv4Addr::new(10, 0, 58, 120), mask, Some(Ipv4Addr::new(10, 0, 58, 1))).is_ok());
        assert!(matches!(
            validate_ip_consistency(Ipv4Addr::new(10, 0, 58, 0), mask, None),
            Err(ValidationError::NetworkOrBroadcastIp)
        ));
        assert!(matches!(
            validate_ip_consistency(Ipv4Addr::new(10, 0, 58, 255), mask, None),
            Err(ValidationError::NetworkOrBroadcastIp)
        ));
        assert!(matches!(
            validate_ip_consistency(Ipv4Addr::new(10, 0, 58, 1), mask, Some(Ipv4Addr::new(10, 0, 58, 1))),
            Err(ValidationError::GatewayEqualsIp)
        ));
    }

    #[test]
    fn test_virtual_filter_keeps_usb_ethernet() {
        assert!(!is_virtual_adapter_name(
            "Ethernet 2",
            "Realtek USB GbE Family Controller"
        ));
        assert!(is_usb_ethernet_hint(
            "Ethernet 2",
            "Realtek USB GbE Family Controller"
        ));
        assert!(!is_virtual_adapter_name(
            "Ethernet 3",
            "ASIX AX88179 USB 3.0 to Gigabit Ethernet Adapter"
        ));
        assert!(!is_virtual_adapter_name(
            "Wi-Fi",
            "Intel(R) Wi-Fi 6 AX201 160MHz"
        ));
        assert!(is_virtual_adapter_name(
            "vEthernet (WSL)",
            "Hyper-V Virtual Ethernet Adapter"
        ));
        assert!(is_virtual_adapter_name(
            "VMware Network Adapter VMnet8",
            "VMware Virtual Ethernet Adapter for VMnet8"
        ));
        assert!(is_virtual_adapter_name(
            "Bluetooth Network Connection",
            "Bluetooth Device (Personal Area Network)"
        ));
        assert!(is_virtual_adapter_name(
            "Local Area Connection* 11",
            "Microsoft Wi-Fi Direct Virtual Adapter"
        ));
        assert!(is_virtual_adapter_name("Tailscale", "Tailscale Tunnel"));
    }

    #[test]
    fn test_sort_and_pick_default_prefers_ethernet() {
        let mut list = vec![
            NetworkAdapter {
                if_index: 5,
                friendly_name: "Wi-Fi".into(),
                description: "Intel Wi-Fi 6".into(),
                kind: NicKind::Wifi,
                is_usb_ethernet: false,
                is_connected: true,
                current_ipv4: Some("10.0.29.88".into()),
            },
            NetworkAdapter {
                if_index: 9,
                friendly_name: "Ethernet".into(),
                description: "Realtek PCIe GbE".into(),
                kind: NicKind::Ethernet,
                is_usb_ethernet: false,
                is_connected: false,
                current_ipv4: None,
            },
            NetworkAdapter {
                if_index: 12,
                friendly_name: "Ethernet 2".into(),
                description: "Realtek USB GbE Family Controller".into(),
                kind: NicKind::Ethernet,
                is_usb_ethernet: true,
                is_connected: true,
                current_ipv4: Some("192.168.1.45".into()),
            },
        ];

        let default_idx = sort_and_pick_default_nic(&mut list);
        assert_eq!(default_idx, Some(0));
        assert!(has_ethernet_adapter(&list));
        assert_eq!(list[0].if_index, 12);
        assert_eq!(list[1].if_index, 9);
        assert_eq!(list[2].if_index, 5);
    }

    #[test]
    fn test_no_ethernet_does_not_auto_select_wifi() {
        let mut list = vec![NetworkAdapter {
            if_index: 5,
            friendly_name: "Wi-Fi".into(),
            description: "Intel(R) Wi-Fi 6 AX201 160MHz".into(),
            kind: NicKind::Wifi,
            is_usb_ethernet: false,
            is_connected: true,
            current_ipv4: Some("10.0.29.88".into()),
        }];

        let default_idx = sort_and_pick_default_nic(&mut list);
        assert_eq!(default_idx, None);
        assert!(!has_ethernet_adapter(&list));
    }

    #[test]
    fn test_build_netsh_commands_with_prefer_wifi() {
        let req = IpConfigRequest {
            adapter_if_index: 12,
            adapter_name: "Ethernet 2".into(),
            ip: Ipv4Addr::new(10, 0, 58, 120),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Some(Ipv4Addr::new(10, 0, 58, 1)),
            dns_primary: Some(Ipv4Addr::new(10, 0, 58, 11)),
            dns_secondary: Some(Ipv4Addr::new(10, 0, 29, 5)),
            priority: Some(PriorityUpdate {
                eth_if_index: 12,
                eth_name: "Ethernet 2".into(),
                wifi_if_index: 5,
                wifi_name: "Wi-Fi".into(),
                mode: PriorityMode::PreferWifi,
            }),
        };

        let cmds = build_netsh_commands(&req);
        assert_eq!(cmds.len(), 5);
        assert_eq!(
            cmds[0],
            vec![
                "interface",
                "ipv4",
                "set",
                "address",
                "name=12",
                "source=static",
                "address=10.0.58.120",
                "mask=255.255.255.0",
                "gateway=10.0.58.1"
            ]
        );
        assert_eq!(
            cmds[1],
            vec![
                "interface",
                "ipv4",
                "set",
                "dnsservers",
                "name=12",
                "source=static",
                "address=10.0.58.11",
                "validate=no"
            ]
        );
        assert_eq!(
            cmds[2],
            vec![
                "interface",
                "ipv4",
                "add",
                "dnsservers",
                "name=12",
                "address=10.0.29.5",
                "index=2",
                "validate=no"
            ]
        );
        assert_eq!(
            cmds[3],
            vec!["interface", "ipv4", "set", "interface", "5", "metric=10"]
        );
        assert_eq!(
            cmds[4],
            vec!["interface", "ipv4", "set", "interface", "12", "metric=50"]
        );
    }

    #[test]
    fn test_build_netsh_commands_windows_default_priority() {
        let req = IpConfigRequest {
            adapter_if_index: 12,
            adapter_name: "Ethernet".into(),
            ip: Ipv4Addr::new(192, 168, 1, 50),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: None,
            dns_primary: Some(Ipv4Addr::new(10, 0, 58, 11)),
            dns_secondary: None,
            priority: Some(PriorityUpdate {
                eth_if_index: 12,
                eth_name: "Ethernet".into(),
                wifi_if_index: 5,
                wifi_name: "Wi-Fi".into(),
                mode: PriorityMode::WindowsDefault,
            }),
        };

        let cmds = build_netsh_commands(&req);
        assert_eq!(cmds.len(), 4);
        assert_eq!(cmds[0].last().unwrap(), "gateway=none");
        assert_eq!(
            cmds[2],
            vec![
                "interface",
                "ipv4",
                "set",
                "interface",
                "12",
                "metric=automatic"
            ]
        );
        assert_eq!(
            cmds[3],
            vec![
                "interface",
                "ipv4",
                "set",
                "interface",
                "5",
                "metric=automatic"
            ]
        );
    }
}
