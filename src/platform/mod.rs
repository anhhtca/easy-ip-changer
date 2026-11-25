use crate::domain::{ApplyReport, IpConfigRequest, NetworkAdapter};

#[cfg(windows)]
mod windows_impl;

#[cfg(not(windows))]
mod mock_impl;

pub trait SystemNetworkBackend: Send + Sync {
    fn is_elevated(&self) -> bool;
    fn relaunch_elevated(&self) -> Result<(), String>;
    fn list_physical_adapters(&self) -> Result<Vec<NetworkAdapter>, String>;
    fn apply_config(&self, req: &IpConfigRequest) -> Result<ApplyReport, String>;
    fn is_preview_mode(&self) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub preview_no_admin: bool,
    pub preview_step2: bool,
    pub preview_single_nic: bool,
    pub preview_wifi_only: bool,
}

impl LaunchOptions {
    pub fn from_args() -> Self {
        let mut opts = Self::default();
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--preview-no-admin" => opts.preview_no_admin = true,
                "--preview-step2" => opts.preview_step2 = true,
                "--preview-single-nic" => opts.preview_single_nic = true,
                "--preview-wifi-only" => opts.preview_wifi_only = true,
                _ => {}
            }
        }
        opts
    }
}

#[cfg(windows)]
pub fn create_backend(_opts: &LaunchOptions) -> Box<dyn SystemNetworkBackend> {
    Box::new(windows_impl::WindowsBackend)
}

#[cfg(not(windows))]
pub fn create_backend(opts: &LaunchOptions) -> Box<dyn SystemNetworkBackend> {
    Box::new(mock_impl::MockBackend::new(opts))
}

pub fn open_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Liên kết không hợp lệ.".to_string());
    }

    #[cfg(windows)]
    {
        windows_impl::open_url(url)
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Không thể mở trình duyệt: {e}"))
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Không thể mở trình duyệt: {e}"))
    }
}
