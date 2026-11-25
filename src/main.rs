#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use slint::{ModelRc, SharedString, VecModel};

use crate::domain::{
    DEFAULT_DNS_PRIMARY, DEFAULT_DNS_SECONDARY, DEFAULT_SUBNET, IpConfigRequest, NetworkAdapter,
    NicKind, PriorityMode, PriorityUpdate, derive_gateway, has_ethernet_adapter,
    parse_pasted_ipv4, resolve_priority_pair, sanitize_octet_digits, sort_and_pick_default_nic,
    validate_ip_consistency, validate_ip_octets, validate_optional_ipv4, validate_subnet_mask,
};
use crate::platform::{LaunchOptions, SystemNetworkBackend, create_backend};

slint::include_modules!();

struct AppState {
    adapters: Vec<NetworkAdapter>,
    gateway_manually_edited: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    let opts = LaunchOptions::from_args();
    let backend: Arc<Box<dyn SystemNetworkBackend>> = Arc::new(create_backend(&opts));
    let state = Rc::new(RefCell::new(AppState {
        adapters: Vec::new(),
        gateway_manually_edited: false,
    }));

    let ui = AppWindow::new()?;
    ui.set_is_preview_mode(backend.is_preview_mode());
    ui.set_subnet(SharedString::from(DEFAULT_SUBNET));
    ui.set_dns_primary(SharedString::from(DEFAULT_DNS_PRIMARY));
    ui.set_dns_secondary(SharedString::from(DEFAULT_DNS_SECONDARY));

    // Helper: quét và đổ danh sách card mạng vật lý vào Màn hình 1
    let load_adapters = {
        let ui_weak = ui.as_weak();
        let backend = Arc::clone(&backend);
        let state = Rc::clone(&state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match backend.list_physical_adapters() {
                Ok(mut adapters) => {
                    let default_idx = sort_and_pick_default_nic(&mut adapters);
                    let has_eth = has_ethernet_adapter(&adapters);

                    let items: Vec<NicItem> = adapters
                        .iter()
                        .map(|a| NicItem {
                            if_index: a.if_index as i32,
                            friendly_name: SharedString::from(a.friendly_name.as_str()),
                            description: SharedString::from(a.description.as_str()),
                            type_label: SharedString::from(
                                a.kind.display_label(a.is_usb_ethernet),
                            ),
                            is_ethernet: a.kind == NicKind::Ethernet,
                            is_connected: a.is_connected,
                            status_text: SharedString::from(a.status_text()),
                            ip_summary: SharedString::from(a.ip_summary()),
                        })
                        .collect();

                    let model = Rc::new(VecModel::from(items));
                    ui.set_adapters(ModelRc::from(model));
                    ui.set_has_ethernet(has_eth);
                    ui.set_selected_nic_index(default_idx.map_or(-1, |i| i as i32));
                    ui.set_scan_error(SharedString::new());
                    state.borrow_mut().adapters = adapters;
                }
                Err(err) => {
                    ui.set_adapters(ModelRc::from(Rc::new(VecModel::<NicItem>::default())));
                    ui.set_has_ethernet(true);
                    ui.set_selected_nic_index(-1);
                    ui.set_scan_error(SharedString::from(err));
                    state.borrow_mut().adapters.clear();
                }
            }
        }
    };

    // Helper: chuẩn bị thông số cho Màn hình 2 dựa trên card đang chọn
    let enter_step_2 = {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let idx = ui.get_selected_nic_index();
            if idx < 0 {
                return;
            }
            let st = state.borrow();
            let Some(adapter) = st.adapters.get(idx as usize) else {
                return;
            };

            ui.set_selected_adapter_name(SharedString::from(adapter.friendly_name.as_str()));
            ui.set_selected_adapter_type(SharedString::from(
                adapter.kind.display_label(adapter.is_usb_ethernet),
            ));
            ui.set_selected_adapter_is_eth(adapter.kind == NicKind::Ethernet);
            ui.set_selected_adapter_current_ip(SharedString::from(adapter.ip_summary()));

            if let Some((eth, wifi)) = resolve_priority_pair(&st.adapters, idx as usize) {
                ui.set_can_change_priority(true);
                ui.set_priority_pair_caption(SharedString::from(format!(
                    "Áp dụng giữa: {} và {}",
                    eth.friendly_name, wifi.friendly_name
                )));
            } else {
                ui.set_can_change_priority(false);
                ui.set_change_priority(false);
                ui.set_priority_pair_caption(SharedString::new());
            }

            ui.set_form_error(SharedString::new());
            ui.set_ip_has_error(false);
            ui.set_current_screen(2);
            ui.invoke_focus_octet(0);
        }
    };

    // Bước (a): Kiểm tra quyền hiện tại khi khởi động
    if !backend.is_elevated() {
        ui.set_current_screen(0);
    } else {
        load_adapters();
        if opts.preview_step2 && !state.borrow().adapters.is_empty() {
            enter_step_2();
        } else {
            ui.set_current_screen(1);
        }
    }

    // Callback: Màn hình 0 — Mở lại với quyền Quản trị
    {
        let ui_weak = ui.as_weak();
        let backend = Arc::clone(&backend);
        let load_adapters = load_adapters.clone();
        ui.on_request_elevation(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match backend.relaunch_elevated() {
                Ok(()) => {
                    // Trong chế độ Preview macOS, chuyển thẳng sang Màn hình 1
                    ui.set_privilege_error(SharedString::new());
                    load_adapters();
                    ui.set_current_screen(1);
                }
                Err(msg) => {
                    ui.set_privilege_error(SharedString::from(msg));
                }
            }
        });
    }

    // Callback: Màn hình 1 — Quét lại danh sách NIC
    {
        let load_adapters = load_adapters.clone();
        ui.on_refresh_adapters(move || {
            load_adapters();
        });
    }

    // Callback: Màn hình 1 — Tiếp tục sang Màn hình 2
    {
        let enter_step_2 = enter_step_2.clone();
        ui.on_proceed_to_ip_config(move || {
            enter_step_2();
        });
    }

    // Callback: Màn hình 2 — Quay lại Màn hình 1
    {
        let ui_weak = ui.as_weak();
        ui.on_back_to_nic_select(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_form_error(SharedString::new());
                ui.set_current_screen(1);
            }
        });
    }

    // Callback: Màn hình 2 — Nhập 4 ô IP thông minh + tự đồng bộ Gateway
    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_octet_edited(move |slot_idx, raw_text| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            ui.set_ip_has_error(false);
            ui.set_form_error(SharedString::new());

            let raw = raw_text.as_str();

            // 1. Hỗ trợ dán nguyên địa chỉ IPv4 (vd "10.0.58.120")
            if let Some([a, b, c, d]) = parse_pasted_ipv4(raw) {
                ui.set_ip1(SharedString::from(a.as_str()));
                ui.set_ip2(SharedString::from(b.as_str()));
                ui.set_ip3(SharedString::from(c.as_str()));
                ui.set_ip4(SharedString::from(d.as_str()));
                sync_gateway_if_auto(&ui, &state.borrow());
                ui.invoke_focus_octet(3);
                return;
            }

            // 2. Xử lý gõ dấu '.' hoặc khoảng trắng để nhảy nhanh sang ô tiếp theo
            let jump_requested = raw.contains('.') || raw.contains(' ');
            let cleaned = sanitize_octet_digits(raw);

            match slot_idx {
                0 => ui.set_ip1(SharedString::from(cleaned.as_str())),
                1 => ui.set_ip2(SharedString::from(cleaned.as_str())),
                2 => ui.set_ip3(SharedString::from(cleaned.as_str())),
                3 => ui.set_ip4(SharedString::from(cleaned.as_str())),
                _ => {}
            }

            sync_gateway_if_auto(&ui, &state.borrow());

            // Tự động nhảy sang ô tiếp theo khi gõ dấu chấm hoặc đã đủ 3 chữ số hợp lệ
            let should_advance = slot_idx < 3
                && ((jump_requested && !cleaned.is_empty())
                    || (cleaned.len() == 3 && cleaned.parse::<u8>().is_ok()));
            if should_advance {
                ui.invoke_focus_octet(slot_idx + 1);
            }
        });
    }

    // Callback: Màn hình 2 — Bật/tắt nút Sửa cho Subnet, Gateway, DNS
    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_subnet_edit(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_subnet_editable(!ui.get_subnet_editable());
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_gateway_edit(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_gateway_editable(!ui.get_gateway_editable());
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_gateway_manual_edited(move |val| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let trimmed = val.trim();
            if trimmed.is_empty() {
                // Nếu người dùng xóa trắng Gateway -> bật lại đồng bộ tự động
                state.borrow_mut().gateway_manually_edited = false;
                ui.set_gateway_auto_synced(true);
            } else {
                state.borrow_mut().gateway_manually_edited = true;
                ui.set_gateway_auto_synced(false);
            }
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_dns_edit(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_dns_editable(!ui.get_dns_editable());
            }
        });
    }

    // Callback: Màn hình 2 — Checkbox "Thay đổi ưu tiên Eth và Wifi"
    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_change_priority(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if !ui.get_can_change_priority() {
                    return;
                }
                let next = !ui.get_change_priority();
                ui.set_change_priority(next);
            }
        });
    }

    // Callback: Màn hình 2 — Nút "Ghi"
    {
        let ui_weak = ui.as_weak();
        let backend = Arc::clone(&backend);
        let state = Rc::clone(&state);
        ui.on_save_config(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let ip1 = ui.get_ip1().to_string();
            let ip2 = ui.get_ip2().to_string();
            let ip3 = ui.get_ip3().to_string();
            let ip4 = ui.get_ip4().to_string();

            let ip = match validate_ip_octets([&ip1, &ip2, &ip3, &ip4]) {
                Ok(addr) => addr,
                Err(err) => {
                    ui.set_ip_has_error(true);
                    ui.set_form_error(SharedString::from(err.to_string()));
                    return;
                }
            };

            let subnet_str = ui.get_subnet().to_string();
            let subnet = match validate_subnet_mask(&subnet_str) {
                Ok(mask) => mask,
                Err(err) => {
                    ui.set_form_error(SharedString::from(err.to_string()));
                    return;
                }
            };

            let gw_str = ui.get_gateway().to_string();
            let gateway = match validate_optional_ipv4(&gw_str, "Default Gateway") {
                Ok(gw) => gw,
                Err(err) => {
                    ui.set_form_error(SharedString::from(err.to_string()));
                    return;
                }
            };

            if let Err(err) = validate_ip_consistency(ip, subnet, gateway) {
                ui.set_ip_has_error(true);
                ui.set_form_error(SharedString::from(err.to_string()));
                return;
            }

            let dns1_str = ui.get_dns_primary().to_string();
            let dns_primary = match validate_optional_ipv4(&dns1_str, "DNS chính") {
                Ok(d) => d,
                Err(err) => {
                    ui.set_form_error(SharedString::from(err.to_string()));
                    return;
                }
            };

            let dns2_str = ui.get_dns_secondary().to_string();
            let dns_secondary = match validate_optional_ipv4(&dns2_str, "DNS phụ") {
                Ok(d) => d,
                Err(err) => {
                    ui.set_form_error(SharedString::from(err.to_string()));
                    return;
                }
            };

            let selected_idx = ui.get_selected_nic_index();
            let (adapter_if_index, adapter_name, priority) = {
                let st = state.borrow();
                let Some(adapter) = st.adapters.get(selected_idx as usize) else {
                    ui.set_form_error(SharedString::from(
                        "Không tìm thấy card mạng đã chọn. Vui lòng quay lại bước 1.",
                    ));
                    return;
                };

                let priority = if ui.get_change_priority() && ui.get_can_change_priority() {
                    resolve_priority_pair(&st.adapters, selected_idx as usize).map(
                        |(eth, wifi)| PriorityUpdate {
                            eth_if_index: eth.if_index,
                            eth_name: eth.friendly_name.clone(),
                            wifi_if_index: wifi.if_index,
                            wifi_name: wifi.friendly_name.clone(),
                            mode: PriorityMode::from_index(ui.get_priority_mode()),
                        },
                    )
                } else {
                    None
                };

                (adapter.if_index, adapter.friendly_name.clone(), priority)
            };

            let request = IpConfigRequest {
                adapter_if_index,
                adapter_name,
                ip,
                subnet,
                gateway,
                dns_primary,
                dns_secondary,
                priority,
            };

            ui.set_saving(true);
            ui.set_form_error(SharedString::new());
            ui.set_ip_has_error(false);

            let ui_weak_bg = ui.as_weak();
            let backend_bg = Arc::clone(&backend);
            std::thread::spawn(move || {
                let result = backend_bg.apply_config(&request);
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(ui) = ui_weak_bg.upgrade() else {
                        return;
                    };
                    ui.set_saving(false);
                    match result {
                        Ok(report) => {
                            ui.set_result_adapter(SharedString::from(report.adapter_name));
                            ui.set_result_ip(SharedString::from(report.ip));
                            ui.set_result_subnet(SharedString::from(report.subnet));
                            ui.set_result_gateway(SharedString::from(report.gateway));
                            ui.set_result_dns1(SharedString::from(report.dns_primary));
                            ui.set_result_dns2(SharedString::from(report.dns_secondary));
                            ui.set_result_priority(SharedString::from(
                                report.priority_summary.unwrap_or_default(),
                            ));
                            ui.set_current_screen(3);
                        }
                        Err(err) => {
                            ui.set_form_error(SharedString::from(err));
                        }
                    }
                });
            });
        });
    }

    // Callback: Màn hình 3 — Sửa lại cấu hình
    {
        let ui_weak = ui.as_weak();
        let load_adapters = load_adapters.clone();
        ui.on_edit_again(move || {
            load_adapters();
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_screen(2);
            }
        });
    }

    // Callback: Màn hình 3 — Đóng ứng dụng
    {
        let ui_weak = ui.as_weak();
        ui.on_close_app(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let _ = ui.hide();
            }
            let _ = slint::quit_event_loop();
        });
    }

    ui.run()
}

fn sync_gateway_if_auto(ui: &AppWindow, state: &AppState) {
    if state.gateway_manually_edited {
        return;
    }
    let gw = derive_gateway(
        ui.get_ip1().as_str(),
        ui.get_ip2().as_str(),
        ui.get_ip3().as_str(),
    );
    ui.set_gateway(SharedString::from(gw.as_str()));
    ui.set_gateway_auto_synced(true);
}

#[cfg(all(test, not(windows)))]
mod ui_snapshot_tests {
    use super::*;
    use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
    use slint::platform::{Platform, WindowAdapter};
    use slint::{PhysicalSize, Rgb8Pixel, WindowSize};
    use std::io::Write;
    use std::path::Path;
    use std::sync::Mutex;
    use std::time::Instant;

    struct HeadlessPlatform {
        window: Rc<MinimalSoftwareWindow>,
        started: Instant,
    }

    impl Platform for HeadlessPlatform {
        fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
            Ok(self.window.clone())
        }

        fn duration_since_start(&self) -> std::time::Duration {
            self.started.elapsed()
        }
    }

    fn dump_frame(
        window: &MinimalSoftwareWindow,
        width: u32,
        height: u32,
        out_path: &Path,
    ) {
        // Đẩy thời gian animation để accordion/transition đạt trạng thái cuối
        slint::platform::update_timers_and_animations();
        let mut buffer = vec![Rgb8Pixel::default(); (width * height) as usize];
        window.request_redraw();
        window.draw_if_needed(|renderer| {
            renderer.render(&mut buffer, width as usize);
        });

        let mut file = std::fs::File::create(out_path).expect("tạo file ppm");
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        let mut bytes = Vec::with_capacity(buffer.len() * 3);
        for px in &buffer {
            bytes.push(px.r);
            bytes.push(px.g);
            bytes.push(px.b);
        }
        file.write_all(&bytes).unwrap();
    }

    #[test]
    fn render_ui_previews_when_requested() {
        let Ok(out_dir) = std::env::var("DUMP_UI_PREVIEWS_DIR") else {
            return;
        };
        static LOCK: Mutex<()> = Mutex::new(());
        let _guard = LOCK.lock().unwrap();

        let width: u32 = 560;
        let height: u32 = 730;
        let sw_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        sw_window.set_size(WindowSize::Physical(PhysicalSize::new(width, height)));

        slint::platform::set_platform(Box::new(HeadlessPlatform {
            window: sw_window.clone(),
            started: Instant::now(),
        }))
        .expect("set headless slint platform");

        let ui = AppWindow::new().expect("create AppWindow");
        ui.set_is_preview_mode(true);
        ui.set_subnet(SharedString::from(DEFAULT_SUBNET));
        ui.set_dns_primary(SharedString::from(DEFAULT_DNS_PRIMARY));
        ui.set_dns_secondary(SharedString::from(DEFAULT_DNS_SECONDARY));

        let backend = create_backend(&LaunchOptions::default());
        let adapters = backend.list_physical_adapters().unwrap();
        let items: Vec<NicItem> = adapters
            .iter()
            .map(|a| NicItem {
                if_index: a.if_index as i32,
                friendly_name: SharedString::from(a.friendly_name.as_str()),
                description: SharedString::from(a.description.as_str()),
                type_label: SharedString::from(a.kind.display_label(a.is_usb_ethernet)),
                is_ethernet: a.kind == NicKind::Ethernet,
                is_connected: a.is_connected,
                status_text: SharedString::from(a.status_text()),
                ip_summary: SharedString::from(a.ip_summary()),
            })
            .collect();
        ui.set_adapters(ModelRc::from(Rc::new(VecModel::from(items))));
        ui.set_selected_nic_index(0);

        let dir = Path::new(&out_dir);
        std::fs::create_dir_all(dir).unwrap();

        // 1. Màn hình 0: Yêu cầu quyền Quản trị
        ui.set_current_screen(0);
        dump_frame(&sw_window, width, height, &dir.join("01_privilege.ppm"));

        // 2. Màn hình 1: Chọn kết nối mạng
        ui.set_current_screen(1);
        dump_frame(&sw_window, width, height, &dir.join("02_nic_select.ppm"));

        // 3. Màn hình 2: Nhập IP (mặc định khóa Subnet/GW/DNS, Gateway tự sinh)
        ui.set_selected_adapter_name(SharedString::from("Ethernet"));
        ui.set_selected_adapter_type(SharedString::from("Ethernet (USB)"));
        ui.set_selected_adapter_is_eth(true);
        ui.set_selected_adapter_current_ip(SharedString::from("IP hiện tại: 192.168.1.45"));
        ui.set_ip1(SharedString::from("10"));
        ui.set_ip2(SharedString::from("0"));
        ui.set_ip3(SharedString::from("58"));
        ui.set_ip4(SharedString::from("120"));
        ui.set_gateway(SharedString::from("10.0.58.1"));
        ui.set_gateway_auto_synced(true);
        ui.set_can_change_priority(true);
        ui.set_priority_pair_caption(SharedString::from("Áp dụng giữa: Ethernet và Wi-Fi"));
        ui.set_change_priority(false);
        ui.set_current_screen(2);
        dump_frame(&sw_window, width, height, &dir.join("03_ip_config.ppm"));

        // 4. Màn hình 2: Mở khóa sửa Gateway + Bật accordion Thay đổi ưu tiên (chiều cao lớn hơn để thấy rõ full form)
        let tall_h: u32 = 900;
        sw_window.set_size(WindowSize::Physical(PhysicalSize::new(width, tall_h)));
        ui.set_gateway_editable(true);
        ui.set_change_priority(true);
        ui.set_priority_mode(0);
        // Kick animation start then advance past 180ms
        slint::platform::update_timers_and_animations();
        std::thread::sleep(std::time::Duration::from_millis(240));
        slint::platform::update_timers_and_animations();
        dump_frame(&sw_window, width, tall_h, &dir.join("04_ip_accordion.ppm"));

        // 5. Màn hình 3: Hoàn tất
        sw_window.set_size(WindowSize::Physical(PhysicalSize::new(width, height)));
        ui.set_result_adapter(SharedString::from("Ethernet"));
        ui.set_result_ip(SharedString::from("10.0.58.120"));
        ui.set_result_subnet(SharedString::from("255.255.255.0"));
        ui.set_result_gateway(SharedString::from("10.0.58.1"));
        ui.set_result_dns1(SharedString::from("10.0.58.11"));
        ui.set_result_dns2(SharedString::from("10.0.29.5"));
        ui.set_result_priority(SharedString::from("Ưu tiên Wi-Fi hơn Ethernet"));
        ui.set_current_screen(3);
        dump_frame(&sw_window, width, height, &dir.join("05_success.ppm"));

        // 6. Màn hình 1 (không có Ethernet — chỉ có Wi-Fi, không tự chọn)
        let wifi_only_backend = create_backend(&LaunchOptions {
            preview_wifi_only: true,
            ..Default::default()
        });
        let mut wifi_adapters = wifi_only_backend.list_physical_adapters().unwrap();
        let default_idx = sort_and_pick_default_nic(&mut wifi_adapters);
        let wifi_items: Vec<NicItem> = wifi_adapters
            .iter()
            .map(|a| NicItem {
                if_index: a.if_index as i32,
                friendly_name: SharedString::from(a.friendly_name.as_str()),
                description: SharedString::from(a.description.as_str()),
                type_label: SharedString::from(a.kind.display_label(a.is_usb_ethernet)),
                is_ethernet: a.kind == NicKind::Ethernet,
                is_connected: a.is_connected,
                status_text: SharedString::from(a.status_text()),
                ip_summary: SharedString::from(a.ip_summary()),
            })
            .collect();
        ui.set_adapters(ModelRc::from(Rc::new(VecModel::from(wifi_items))));
        ui.set_has_ethernet(has_ethernet_adapter(&wifi_adapters));
        ui.set_selected_nic_index(default_idx.map_or(-1, |i| i as i32));
        ui.set_current_screen(1);
        dump_frame(
            &sw_window,
            width,
            height,
            &dir.join("06_no_ethernet_wifi_only.ppm"),
        );
    }
}
