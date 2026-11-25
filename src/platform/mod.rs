use std::path::{Path, PathBuf};

use crate::domain::{ApplyReport, IpConfigRequest, NetworkAdapter};

pub const USER_GUIDE_FILENAME: &str = "Huong-dan-su-dung.pdf";

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
    pub uac_relaunched: bool,
    pub preview_step2: bool,
    pub preview_single_nic: bool,
    pub preview_wifi_only: bool,
}

impl LaunchOptions {
    pub fn from_args() -> Self {
        let mut opts = Self::default();
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--uac-relaunched" => opts.uac_relaunched = true,
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

/// Tìm file PDF hướng dẫn sử dụng (`Huong-dan-su-dung.pdf`) đặt cùng thư mục với file `.exe`.
/// Trong chế độ phát triển / xem trước trên macOS, có hỗ trợ kiểm tra thêm thư mục `docs/` của dự án.
pub fn find_user_guide_path() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
        && let Some(found) = find_user_guide_in_dir(exe_dir)
    {
        return Some(found);
    }

    #[cfg(any(not(windows), debug_assertions))]
    {
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(found) = find_user_guide_in_dir(&cwd) {
                return Some(found);
            }
            if let Some(found) = find_user_guide_in_dir(&cwd.join("docs")) {
                return Some(found);
            }
        }
        let manifest_docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
        if let Some(found) = find_user_guide_in_dir(&manifest_docs) {
            return Some(found);
        }
    }

    None
}

pub fn find_user_guide_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidate = dir.join(USER_GUIDE_FILENAME);
    candidate.is_file().then_some(candidate)
}

pub fn open_local_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("Không tìm thấy tệp tài liệu hướng dẫn.".to_string());
    }

    #[cfg(windows)]
    {
        windows_impl::open_file(path)
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Không thể mở tài liệu hướng dẫn: {e}"))
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Không thể mở tài liệu hướng dẫn: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_guide_only_when_pdf_exists_in_dir() {
        let base = std::env::temp_dir().join(format!(
            "easy-ip-changer-guide-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        assert!(find_user_guide_in_dir(&base).is_none());

        let pdf_path = base.join(USER_GUIDE_FILENAME);
        std::fs::write(&pdf_path, b"%PDF-1.4\n").unwrap();
        assert_eq!(find_user_guide_in_dir(&base), Some(pdf_path));

        let _ = std::fs::remove_dir_all(&base);
    }
}
