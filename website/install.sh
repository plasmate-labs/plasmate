#!/bin/sh
# Plasmate installer
# Usage: curl -fsSL https://plasmate.app/install.sh | sh
#
# Downloads the latest Plasmate binary for your platform.
# Installs to /usr/local/bin (or ~/.local/bin if no sudo).

set -e

REPO="plasmate-labs/plasmate"
BINARY="plasmate"
TAG="${PLASMATE_VERSION:-v0.1.1}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { printf "${CYAN}[plasmate]${NC} %s\n" "$1"; }
success() { printf "${GREEN}[plasmate]${NC} %s\n" "$1"; }
error() { printf "${RED}[plasmate]${NC} %s\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported OS: $(uname -s). Plasmate supports Linux and macOS." ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              error "Unsupported architecture: $(uname -m). Plasmate supports x86_64 and arm64." ;;
    esac
}

# Find install directory
find_install_dir() {
    if [ -w /usr/local/bin ]; then
        echo "/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        echo "$HOME/.local/bin"
    else
        mkdir -p "$HOME/.local/bin"
        echo "$HOME/.local/bin"
    fi
}

# Download
download() {
    local url="$1" dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$dest" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        error "Neither curl nor wget found. Install one and retry."
    fi
}

main() {
    local os arch artifact install_dir url

    os=$(detect_os)
    arch=$(detect_arch)
    artifact="${BINARY}-${arch}-${os}"

    info "Detected platform: ${os}/${arch}"
    info "Installing Plasmate ${TAG}..."

    url="https://github.com/${REPO}/releases/download/${TAG}/${artifact}"

    # Download to temp
    local tmp
    tmp=$(mktemp)
    info "Downloading ${url}..."
    download "$url" "$tmp" || error "Download failed. Check that release ${TAG} exists at github.com/${REPO}/releases"

    # Install
    install_dir=$(find_install_dir)
    chmod +x "$tmp"
    mv "$tmp" "${install_dir}/${BINARY}"

    success "Installed ${BINARY} to ${install_dir}/${BINARY}"

    # Verify
    if command -v plasmate >/dev/null 2>&1; then
        local version
        version=$(plasmate --version 2>/dev/null || echo "unknown")
        success "Version: ${version}"
    else
        info "Add ${install_dir} to your PATH:"
        info "  export PATH=\"${install_dir}:\$PATH\""
    fi

    echo ""
    info "Quick start:"
    info "  plasmate fetch https://plasmate.app      # One-shot SOM output"
    info "  plasmate serve                            # Start AWP server on :9222"
    info "  docker run -p 9222:9222 plasmate/browser  # Docker"
    echo ""
    success "Done! Docs: https://github.com/plasmate-labs/plasmate"
}

main "$@"
