#!/bin/bash
set -e

#===============================================================================
# masmide - Remote Installer
# Downloads the latest release and runs the bundled installer.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash
#
# Or with a specific version:
#   curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash -s -- v0.2.0
#===============================================================================

VERSION="${1:-latest}"
REPO="KM-Alee/masmide"
TMP_DIR=$(mktemp -d)

#-------------------------------------------------------------------------------
# Colors
#-------------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}[+]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[x]${NC} $1"; cleanup; exit 1; }
step()    { echo -e "${BLUE}[>]${NC} ${BOLD}$1${NC}"; }

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

#-------------------------------------------------------------------------------
# System detection
#-------------------------------------------------------------------------------
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              error "Unsupported architecture: $(uname -m)" ;;
    esac
}

#-------------------------------------------------------------------------------
# Download helpers
#-------------------------------------------------------------------------------
get_latest_version() {
    curl -sSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

download_release() {
    local version="$1" arch="$2"
    local url="https://github.com/$REPO/releases/download/${version}/masmide-${version}-linux-${arch}.tar.gz"

    step "Downloading masmide $version for $arch..."

    if ! curl -sSL --fail -o "$TMP_DIR/masmide.tar.gz" "$url"; then
        error "Download failed: $url"
    fi

    info "Downloaded successfully"
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    echo
    echo -e "${BOLD}  masmide remote installer${NC}"
    echo

    # Pre-flight
    for cmd in curl tar; do
        command -v "$cmd" &>/dev/null || error "'$cmd' is required but not installed."
    done

    local arch
    arch=$(detect_arch)
    info "Architecture: $arch"

    # Resolve version
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
        [ -z "$VERSION" ] && error "Could not determine latest version. Specify one: bash -s -- v0.2.0"
    fi
    info "Version: $VERSION"

    echo
    echo "  This will download and install:"
    echo "    - masmide $VERSION (with bundled JWasm)"
    echo "    - Irvine32 library"
    echo "    - System packages: mingw-w64, wine"
    echo
    read -p "  Proceed? [Y/n] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Nn]$ ]] && exit 0

    echo

    # Download and extract
    download_release "$VERSION" "$arch"
    step "Extracting..."
    tar -xzf "$TMP_DIR/masmide.tar.gz" -C "$TMP_DIR"

    # Find the extracted directory
    local extracted
    extracted=$(find "$TMP_DIR" -maxdepth 1 -type d -name "masmide-*" | head -1)
    if [ -z "$extracted" ]; then
        extracted="$TMP_DIR"
    fi

    # Hand off to the bundled installer
    if [ -f "$extracted/install.sh" ]; then
        info "Running bundled installer..."
        echo
        chmod +x "$extracted/install.sh"
        bash "$extracted/install.sh"
    else
        error "install.sh not found in release archive"
    fi
}

main "$@"
