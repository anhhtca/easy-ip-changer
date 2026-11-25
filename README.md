# Easy IP Changer (Windows 10/11 Pro)

Công cụ đổi nhanh địa chỉ IP tĩnh và thứ tự ưu tiên giữa Ethernet / Wi-Fi trên **Windows 10/11 Pro**, viết hoàn toàn bằng **Rust + Slint**, đóng gói thành **1 file `.exe` duy nhất, không phụ thuộc runtime ngoài (zero-dependency)**, giao diện **100% tiếng Việt** thân thiện cho người dùng cuối.

## Tính năng chính

1. **Tự động yêu cầu quyền Quản trị (UAC) khi mở ứng dụng:** Manifest nhúng sẵn `requireAdministrator` giúp Windows hỏi quyền UAC ngay khi khởi chạy (không cần màn hình thông báo mở lại) và vào thẳng bước chọn kết nối mạng.
2. **Chọn kết nối mạng vật lý:** Chỉ liệt kê **Ethernet** (bao gồm cả **USB Ethernet**) và **Wi-Fi**, tự lọc bỏ card ảo (VPN, Hyper-V, VMware, VirtualBox, Bluetooth, Wi-Fi Direct...). Ethernet được làm nổi bật, chọn mặc định và gắn nhãn **Nên chọn**.
3. **Nhập IP 4 ô thông minh:**
   - 4 ô nhập số tách biệt (`0–255`), gõ dấu chấm `.` hoặc khoảng trắng để tự nhảy ô, hoặc dán (`Ctrl+V`) nguyên địa chỉ IP (ví dụ `10.0.58.120`).
   - **Subnet Mask** mặc định `255.255.255.0`.
   - **Default Gateway** tự động lấy 3 nhóm đầu của IP + `.1` theo thời gian thực.
   - **Máy chủ DNS** mặc định `10.0.58.11` và `10.0.29.5`.
   - Cả 3 nhóm trên mặc định ở chế độ **chỉ đọc (Read-only)** để tránh gõ nhầm, mỗi nhóm đều có nút nhỏ **Sửa** để mở khóa khi cần.
4. **Thay đổi ưu tiên Ethernet & Wi-Fi:**
   - Tick **"Thay đổi ưu tiên Eth và Wifi"** để mở khu vực chọn bằng ngôn ngữ đời thường (**Ưu tiên Wi-Fi** *(mặc định)*, **Ưu tiên Ethernet**, hoặc **Mặc định Windows**), ẩn hoàn toàn con số metric kỹ thuật.
5. **Ghi đồng thời IP và thứ tự ưu tiên:** Thực thi ngầm không chớp cửa sổ đen CMD và hiển thị bảng tóm tắt kết quả rõ ràng.
6. **Tài liệu hướng dẫn PDF đi kèm (`Huong-dan-su-dung.pdf`):** Khi đặt `Huong-dan-su-dung.pdf` cùng thư mục với `easy-ip-changer.exe`, ứng dụng tự động hiển thị khung gợi ý mở hướng dẫn ngay ở Màn hình 1 và nút **Hướng dẫn** trên thanh tiêu đề để mở trực tiếp bằng trình đọc PDF mặc định của Windows. Nếu chỉ copy riêng file `.exe` mà không kèm PDF, các nút hướng dẫn sẽ tự động ẩn gọn gàng.

## Tài liệu hướng dẫn sử dụng (PDF)

- File PDF dành cho người dùng cuối: `docs/Huong-dan-su-dung.pdf` (4 trang A4 kèm ảnh minh họa từng bước, được tự động đóng gói chung cùng `.exe` trong `easy-ip-changer-windows-x64.zip`).
- Để tạo lại ảnh chụp màn hình và xuất lại file PDF sau khi sửa giao diện hoặc nội dung (`docs/guide-source/guide.html`):

```bash
./scripts/build-guide.sh
```

## Build file `.exe` standalone trên Windows 10/11 Pro

Yêu cầu máy build có cài [Rust (MSVC toolchain)](https://rustup.rs/) và Visual Studio C++ Build Tools:

```powershell
cargo build --release
```

File thực thi độc lập nằm tại:

```text
target\release\easy-ip-changer.exe
```

File `.cargo/config.toml` đã cấu hình sẵn `-C target-feature=+crt-static` và Slint dùng software renderer tích hợp sẵn, nên file `.exe` có thể copy sang bất kỳ máy Windows 10/11 Pro nào và chạy trực tiếp mà **không cần cài .NET, WebView2 hay VC++ Redistributable**.

## Build release Windows `.exe` ngay trên macOS (Cross-compile)

Để build file `.exe` kiểm thử nhanh trên máy Windows mà không cần đợi GitHub Actions, dùng script `scripts/build-windows.sh` (sử dụng `cargo-xwin` + `LLVM/LLD`, cùng target `x86_64-pc-windows-msvc` và static CRT giống hệt CI):

```bash
# Lần đầu (tự cài LLVM, LLD, Rust target x86_64-pc-windows-msvc và cargo-xwin rồi build):
./scripts/build-windows.sh --setup

# Các lần build tiếp theo:
./scripts/build-windows.sh

# Chỉ build nhanh file .exe (bỏ qua đóng gói .zip và SHA256SUMS.txt):
./scripts/build-windows.sh --no-zip
```

File đầu ra nằm trong thư mục `dist/`:

- `dist/easy-ip-changer.exe` (đã nhúng sẵn icon, manifest và static CRT)
- `dist/Huong-dan-su-dung.pdf` (tài liệu hướng dẫn sử dụng cho người dùng cuối)
- `dist/easy-ip-changer-windows-x64.zip` (chứa cả `.exe` và `Huong-dan-su-dung.pdf`)
- `dist/SHA256SUMS.txt`

## Chạy chế độ xem trước (Preview) trên macOS

Trên macOS, ứng dụng tự dùng dữ liệu card mạng giả lập (không thay đổi mạng máy thật):

```bash
# Chạy bình thường (từ Màn hình 1: Chọn kết nối mạng)
cargo run

# Mở thẳng Màn hình 2 (Nhập địa chỉ IP & Ưu tiên)
cargo run -- --preview-step2

# Giả lập máy bàn chỉ có 1 card Ethernet (không có Wi-Fi)
cargo run -- --preview-single-nic

# Giả lập laptop chỉ có Wi-Fi (không có Ethernet -> hiện cảnh báo và không tự chọn Wi-Fi)
cargo run -- --preview-wifi-only
```

## Chạy kiểm thử tự động

```bash
cargo test
```
