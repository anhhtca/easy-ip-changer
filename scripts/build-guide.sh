#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SHOTS_DIR="${REPO_ROOT}/docs/screenshots"
GUIDE_HTML="${REPO_ROOT}/docs/guide-source/guide.html"
GUIDE_PDF="${REPO_ROOT}/docs/Huong-dan-su-dung.pdf"

cd "${REPO_ROOT}"

echo "==> 1. Render ảnh chụp màn hình HiDPI (2x) từ giao diện thật..."
mkdir -p "${SHOTS_DIR}"
DUMP_UI_PREVIEWS_DIR="${SHOTS_DIR}" \
DUMP_UI_PREVIEWS_HIDPI=1 \
DUMP_UI_PREVIEWS_PRODUCTION=1 \
cargo test ui_snapshot_tests::render_ui_previews_when_requested -- --nocapture

python3 - <<'PY'
from PIL import Image
import glob, os

shots_dir = "docs/screenshots"
for ppm in sorted(glob.glob(os.path.join(shots_dir, "*.ppm"))):
    png = ppm[:-4] + ".png"
    Image.open(ppm).save(png, optimize=True)
    os.remove(ppm)

# Cắt gọn phần khung accordion "Thay đổi ưu tiên Eth và Wifi" để minh họa trên Trang 3
acc = Image.open(os.path.join(shots_dir, "04_ip_accordion.png"))
w, h = acc.size
# Khung card ưu tiên nằm từ ~y=1140 đến ~y=1726 (theo tỉ lệ ảnh 1120x1936)
crop_box = (
    int(w * 0.032),
    int(h * 0.592),
    int(w * 0.968),
    int(h * 0.896),
)
acc.crop(crop_box).save(os.path.join(shots_dir, "04_priority_crop.png"), optimize=True)
print(f"Đã tạo ảnh screenshots + 04_priority_crop.png ({acc.crop(crop_box).size})")
PY

echo "==> 2. Xuất tài liệu PDF A4 từ HTML bằng Chrome headless..."
CHROME_BIN=""
for candidate in \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" \
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge" \
  "$(command -v google-chrome || true)" \
  "$(command -v chromium || true)"; do
  if [[ -n "${candidate}" && -x "${candidate}" ]]; then
    CHROME_BIN="${candidate}"
    break
  fi
done

if [[ -z "${CHROME_BIN}" ]]; then
  echo "Lỗi: Không tìm thấy Google Chrome / Chromium để xuất PDF." >&2
  exit 1
fi

CHROME_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/easy-ip-changer-chrome.XXXXXX")"
trap 'rm -rf "${CHROME_TMP_DIR}"' EXIT

"${CHROME_BIN}" \
  --headless=new \
  --disable-gpu \
  --no-first-run \
  --user-data-dir="${CHROME_TMP_DIR}" \
  --no-pdf-header-footer \
  --print-to-pdf="${GUIDE_PDF}" \
  "file://${GUIDE_HTML}"

echo "==> Hoàn tất: ${GUIDE_PDF} ($(du -h "${GUIDE_PDF}" | awk '{print $1}'))"
