use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::domain::{
    ApplyReport, IpConfigRequest, NetworkAdapter, NicKind, build_apply_report,
    sort_and_pick_default_nic,
};
use crate::platform::{LaunchOptions, SystemNetworkBackend};

pub struct MockBackend {
    elevated: AtomicBool,
    single_nic: bool,
    wifi_only: bool,
}

impl MockBackend {
    pub fn new(opts: &LaunchOptions) -> Self {
        Self {
            elevated: AtomicBool::new(!opts.preview_no_admin),
            single_nic: opts.preview_single_nic,
            wifi_only: opts.preview_wifi_only,
        }
    }
}

impl SystemNetworkBackend for MockBackend {
    fn is_elevated(&self) -> bool {
        self.elevated.load(Ordering::Relaxed)
    }

    fn relaunch_elevated(&self) -> Result<(), String> {
        // Trên macOS Preview: mô phỏng việc bấm Yes trên hộp thoại UAC thành công.
        self.elevated.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn list_physical_adapters(&self) -> Result<Vec<NetworkAdapter>, String> {
        let mut adapters = Vec::new();

        if self.wifi_only {
            adapters.push(NetworkAdapter {
                if_index: 5,
                friendly_name: "Wi-Fi".to_string(),
                description: "Intel(R) Wi-Fi 6 AX201 160MHz".to_string(),
                kind: NicKind::Wifi,
                is_usb_ethernet: false,
                is_connected: true,
                current_ipv4: Some("10.0.29.88".to_string()),
            });
            sort_and_pick_default_nic(&mut adapters);
            return Ok(adapters);
        }

        adapters.push(NetworkAdapter {
            if_index: 12,
            friendly_name: "Ethernet".to_string(),
            description: "Realtek USB GbE Family Controller".to_string(),
            kind: NicKind::Ethernet,
            is_usb_ethernet: true,
            is_connected: true,
            current_ipv4: Some("192.168.1.45".to_string()),
        });

        if !self.single_nic {
            adapters.push(NetworkAdapter {
                if_index: 9,
                friendly_name: "Ethernet 2".to_string(),
                description: "Intel(R) Ethernet Connection I219-LM".to_string(),
                kind: NicKind::Ethernet,
                is_usb_ethernet: false,
                is_connected: false,
                current_ipv4: None,
            });
            adapters.push(NetworkAdapter {
                if_index: 5,
                friendly_name: "Wi-Fi".to_string(),
                description: "Intel(R) Wi-Fi 6 AX201 160MHz".to_string(),
                kind: NicKind::Wifi,
                is_usb_ethernet: false,
                is_connected: true,
                current_ipv4: Some("10.0.29.88".to_string()),
            });
        }

        sort_and_pick_default_nic(&mut adapters);
        Ok(adapters)
    }

    fn apply_config(&self, req: &IpConfigRequest) -> Result<ApplyReport, String> {
        // Mô phỏng thời gian ghi cấu hình Windows (~450ms) để kiểm tra trạng thái loading.
        std::thread::sleep(Duration::from_millis(450));
        Ok(build_apply_report(req))
    }

    fn is_preview_mode(&self) -> bool {
        true
    }
}
