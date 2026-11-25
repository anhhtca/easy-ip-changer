#!/usr/bin/env bash
set -euo pipefail

TARGET="x86_64-pc-windows-msvc"
APP_NAME="easy-ip-changer"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DO_SETUP=false
SETUP_ONLY=false
SKIP_ZIP=false

usage() {
  cat <<'EOF'
Build file release Windows (.exe) trực tiếp trên macOS/Linux (dùng cargo-xwin + LLVM).

Cách dùng:
  ./scripts/build-windows.sh [tùy chọn]

Tùy chọn:
  --setup       Tự động cài các công cụ còn thiếu (LLVM, cargo-xwin, Rust Windows target) rồi build
  --setup-only  Chỉ cài các công cụ cần thiết, chưa build
  --no-zip      Chỉ xuất dist/easy-ip-changer.exe (bỏ qua file .zip và SHA256SUMS.txt)
  -h, --help    Hiển thị hướng dẫn này

Kết quả mặc định nằm trong thư mục dist/:
  - dist/easy-ip-changer.exe
  - dist/Huong-dan-su-dung.pdf
  - dist/easy-ip-changer-windows-x64.zip (gồm cả .exe và file PDF hướng dẫn)
  - dist/SHA256SUMS.txt
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --setup)
      DO_SETUP=true
      shift
      ;;
    --setup-only)
      DO_SETUP=true
      SETUP_ONLY=true
      shift
      ;;
    --no-zip)
      SKIP_ZIP=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Lỗi: Tùy chọn không hợp lệ: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

add_llvm_to_path() {
  local candidates=()
  if command -v brew >/dev/null 2>&1; then
    local brew_llvm brew_lld
    brew_llvm="$(brew --prefix llvm 2>/dev/null || true)"
    brew_lld="$(brew --prefix lld 2>/dev/null || true)"
    if [[ -n "${brew_llvm}" && -d "${brew_llvm}/bin" ]]; then
      candidates+=("${brew_llvm}/bin")
    fi
    if [[ -n "${brew_lld}" && -d "${brew_lld}/bin" ]]; then
      candidates+=("${brew_lld}/bin")
    fi
  fi
  candidates+=(
    "/opt/homebrew/opt/llvm/bin"
    "/opt/homebrew/opt/lld/bin"
    "/usr/local/opt/llvm/bin"
    "/usr/local/opt/lld/bin"
  )

  for dir in "${candidates[@]}"; do
    if [[ -d "${dir}" && ":${PATH}:" != *":${dir}:"* ]]; then
      export PATH="${dir}:${PATH}"
    fi
  done

  if [[ -z "${RC_PATH:-}" ]] && command -v llvm-rc >/dev/null 2>&1; then
    export RC_PATH="$(command -v llvm-rc)"
  fi
}

has_rust_target() {
  rustup target list --installed | grep -qx "${TARGET}"
}

has_cargo_xwin() {
  cargo xwin --version >/dev/null 2>&1
}

has_llvm_tools() {
  command -v llvm-rc >/dev/null 2>&1 && command -v clang-cl >/dev/null 2>&1
}

has_lld_tools() {
  command -v lld-link >/dev/null 2>&1
}

install_missing_tools() {
  local brew_pkgs=()
  if ! has_llvm_tools; then
    brew_pkgs+=("llvm")
  fi
  if ! has_lld_tools; then
    brew_pkgs+=("lld")
  fi

  if [[ ${#brew_pkgs[@]} -gt 0 ]]; then
    if [[ "$(uname -s)" == "Darwin" ]] && command -v brew >/dev/null 2>&1; then
      echo "==> Đang cài ${brew_pkgs[*]} qua Homebrew..."
      brew install "${brew_pkgs[@]}"
      add_llvm_to_path
    else
      echo "Lỗi: Thiếu LLVM/LLD (lld-link, clang-cl, llvm-rc). Vui lòng cài thủ công." >&2
      exit 1
    fi
  fi

  if ! has_rust_target; then
    echo "==> Đang thêm Rust target ${TARGET}..."
    rustup target add "${TARGET}"
  fi

  if ! has_cargo_xwin; then
    echo "==> Đang cài cargo-xwin..."
    cargo install --locked cargo-xwin
  fi
}

check_prerequisites() {
  local missing=()
  if ! has_llvm_tools; then
    missing+=("LLVM (brew install llvm)")
  fi
  if ! has_lld_tools; then
    missing+=("LLD (brew install lld)")
  fi
  if ! has_rust_target; then
    missing+=("Rust target ${TARGET} (rustup target add ${TARGET})")
  fi
  if ! has_cargo_xwin; then
    missing+=("cargo-xwin (cargo install --locked cargo-xwin)")
  fi

  if [[ ${#missing[@]} -gt 0 ]]; then
    echo "Thiếu công cụ cần thiết để cross-compile sang Windows:" >&2
    for item in "${missing[@]}"; do
      echo "  - ${item}" >&2
    done
    echo "" >&2
    echo "Chạy lệnh sau để tự động cài đặt và build:" >&2
    echo "  ./scripts/build-windows.sh --setup" >&2
    exit 1
  fi
}

compute_sha256() {
  local file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file}" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  else
    echo "Lỗi: Không tìm thấy shasum hoặc sha256sum." >&2
    exit 1
  fi
}

add_llvm_to_path

if [[ "${DO_SETUP}" == "true" ]]; then
  install_missing_tools
  add_llvm_to_path
fi

check_prerequisites

if [[ "${SETUP_ONLY}" == "true" ]]; then
  echo "Đã cài xong đầy đủ công cụ build Windows."
  exit 0
fi

cd "${REPO_ROOT}"

echo "==> Đang build release Windows (${TARGET})..."
echo "    (Lần đầu chạy sẽ tự tải cache Windows SDK/CRT ~250MB từ xwin)"
cargo xwin build --release --locked --target "${TARGET}" --xwin-arch x86_64

EXE_SOURCE="${REPO_ROOT}/target/${TARGET}/release/${APP_NAME}.exe"
GUIDE_SOURCE="${REPO_ROOT}/docs/Huong-dan-su-dung.pdf"
DIST_DIR="${REPO_ROOT}/dist"
EXE_DEST="${DIST_DIR}/${APP_NAME}.exe"
GUIDE_DEST="${DIST_DIR}/Huong-dan-su-dung.pdf"
ZIP_DEST="${DIST_DIR}/${APP_NAME}-windows-x64.zip"
CHECKSUM_DEST="${DIST_DIR}/SHA256SUMS.txt"

if [[ ! -f "${EXE_SOURCE}" ]]; then
  echo "Lỗi: Không tìm thấy file build tại ${EXE_SOURCE}" >&2
  exit 1
fi

if [[ ! -f "${GUIDE_SOURCE}" ]]; then
  echo "Lỗi: Không tìm thấy tài liệu hướng dẫn tại ${GUIDE_SOURCE}" >&2
  exit 1
fi

mkdir -p "${DIST_DIR}"
cp -f "${EXE_SOURCE}" "${EXE_DEST}"
cp -f "${GUIDE_SOURCE}" "${GUIDE_DEST}"

if [[ "${SKIP_ZIP}" == "false" ]]; then
  rm -f "${ZIP_DEST}"
  zip -j -q "${ZIP_DEST}" "${EXE_DEST}" "${GUIDE_DEST}"

  EXE_HASH="$(compute_sha256 "${EXE_DEST}")"
  GUIDE_HASH="$(compute_sha256 "${GUIDE_DEST}")"
  ZIP_HASH="$(compute_sha256 "${ZIP_DEST}")"
  printf "%s  %s\n%s  %s\n%s  %s\n" \
    "${EXE_HASH}" "$(basename "${EXE_DEST}")" \
    "${GUIDE_HASH}" "$(basename "${GUIDE_DEST}")" \
    "${ZIP_HASH}" "$(basename "${ZIP_DEST}")" \
    > "${CHECKSUM_DEST}"
fi

EXE_SIZE="$(du -h "${EXE_DEST}" | awk '{print $1}')"
GUIDE_SIZE="$(du -h "${GUIDE_DEST}" | awk '{print $1}')"
echo ""
echo "==> Build thành công!"
echo "    EXE : ${EXE_DEST} (${EXE_SIZE})"
echo "    PDF : ${GUIDE_DEST} (${GUIDE_SIZE})"
if [[ "${SKIP_ZIP}" == "false" ]]; then
  echo "    ZIP : ${ZIP_DEST}"
  echo "    SHA : ${CHECKSUM_DEST}"
fi
