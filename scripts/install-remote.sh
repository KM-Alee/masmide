#!/bin/bash
set -e

#═══════════════════════════════════════════════════════════════════════════════
# masmide - Remote Installer
# One-liner installation script for curl | bash
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash
#
# Or with a specific version:
#   curl -sSL https://raw.githubusercontent.com/KM-Alee/masmide/main/scripts/install-remote.sh | bash -s -- v0.1.0
#═══════════════════════════════════════════════════════════════════════════════

VERSION="${1:-latest}"
REPO="KM-Alee/masmide"
INSTALL_DIR="/usr/local/bin"
LIB_DIR="/usr/local/lib/irvine"
INC_DIR="/usr/local/include/irvine"
TMP_DIR=$(mktemp -d)

#───────────────────────────────────────────────────────────────────────────────
# Colors
#───────────────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}[✓]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[✗]${NC} $1"; cleanup; exit 1; }
step()    { echo -e "${BLUE}[→]${NC} ${BOLD}$1${NC}"; }

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

#───────────────────────────────────────────────────────────────────────────────
# System detection
#───────────────────────────────────────────────────────────────────────────────
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              error "Unsupported architecture: $arch" ;;
    esac
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO_ID="$ID"
    elif [ -f /etc/arch-release ]; then
        DISTRO_ID="arch"
    elif [ -f /etc/debian_version ]; then
        DISTRO_ID="debian"
    else
        DISTRO_ID="unknown"
    fi

    case "$DISTRO_ID" in
        arch|manjaro|endeavouros|garuda)
            DISTRO_FAMILY="arch"
            ;;
        debian|ubuntu|linuxmint|pop|elementary|zorin|kali)
            DISTRO_FAMILY="debian"
            ;;
        *)
            DISTRO_FAMILY="unknown"
            ;;
    esac
}

#───────────────────────────────────────────────────────────────────────────────
# Dependency check
#───────────────────────────────────────────────────────────────────────────────
check_command() {
    command -v "$1" &>/dev/null
}

require_command() {
    if ! check_command "$1"; then
        error "$1 is required but not installed. Please install it first."
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Download helpers
#───────────────────────────────────────────────────────────────────────────────
get_latest_version() {
    curl -sSL "https://api.github.com/repos/$REPO/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

download_release() {
    local version="$1"
    local arch="$2"
    local url="https://github.com/$REPO/releases/download/${version}/masmide-${version}-linux-${arch}.tar.gz"
    
    step "Downloading masmide $version for $arch..."
    
    if ! curl -sSL --fail -o "$TMP_DIR/masmide.tar.gz" "$url"; then
        error "Failed to download from $url"
    fi
    
    info "Downloaded successfully"
}

#───────────────────────────────────────────────────────────────────────────────
# Installation
#───────────────────────────────────────────────────────────────────────────────
install_dependencies() {
    step "Checking dependencies..."
    
    local missing=""
    check_command jwasm || missing="$missing jwasm"
    check_command wine || missing="$missing wine"
    (check_command i686-w64-mingw32-ld || check_command x86_64-w64-mingw32-ld) || missing="$missing mingw-w64"
    
    if [ -n "$missing" ]; then
        warn "Missing dependencies:$missing"
        echo
        
        read -p "Install missing dependencies? [Y/n] " -n 1 -r
        echo
        [[ $REPLY =~ ^[Nn]$ ]] && return
        
        case "$DISTRO_FAMILY" in
            arch)
                step "Installing via pacman..."
                sudo pacman -Sy --noconfirm --needed mingw-w64-gcc wine base-devel git
                
                # JWasm from AUR
                if ! check_command jwasm; then
                    if check_command yay; then
                        yay -S --noconfirm jwasm
                    elif check_command paru; then
                        paru -S --noconfirm jwasm
                    else
                        warn "No AUR helper found. Please install jwasm manually."
                    fi
                fi
                ;;
            debian)
                step "Installing via apt..."
                sudo dpkg --add-architecture i386 2>/dev/null || true
                sudo apt-get update -qq
                sudo apt-get install -y mingw-w64 wine64 wine32 build-essential git
                
                # Build JWasm from source
                if ! check_command jwasm; then
                    step "Building JWasm from source..."
                    git clone --depth 1 https://github.com/JWasm/JWasm.git "$TMP_DIR/jwasm"
                    cd "$TMP_DIR/jwasm"
                    make -f GccUnix.mak
                    sudo cp build/GccUnixR/jwasm /usr/local/bin/
                    cd - >/dev/null
                fi
                ;;
            *)
                warn "Unknown distro. Please install dependencies manually:"
                echo "  - mingw-w64 (linker)"
                echo "  - wine"
                echo "  - jwasm (from https://github.com/JWasm/JWasm)"
                ;;
        esac
    else
        info "All dependencies found"
    fi
}

install_masmide() {
    step "Installing masmide..."
    
    # Extract
    tar -xzf "$TMP_DIR/masmide.tar.gz" -C "$TMP_DIR"
    
    # Install binary
    sudo cp "$TMP_DIR/masmide" "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/masmide"
    info "Installed masmide to $INSTALL_DIR/masmide"
    
    # Install Irvine library if present
    if [ -d "$TMP_DIR/Irvine" ]; then
        step "Installing Irvine32 library..."
        sudo mkdir -p "$LIB_DIR" "$INC_DIR"
        sudo cp -f "$TMP_DIR"/Irvine/*.lib "$LIB_DIR/" 2>/dev/null || true
        sudo cp -f "$TMP_DIR"/Irvine/*.Lib "$LIB_DIR/" 2>/dev/null || true
        sudo cp -f "$TMP_DIR"/Irvine/*.obj "$LIB_DIR/" 2>/dev/null || true
        sudo cp -f "$TMP_DIR"/Irvine/*.inc "$INC_DIR/" 2>/dev/null || true
        info "Installed Irvine32 library"
    fi
    
    # Install templates
    if [ -d "$TMP_DIR/templates" ]; then
        local template_dir="$HOME/.config/masmide/templates"
        mkdir -p "$template_dir"
        cp -r "$TMP_DIR/templates/"* "$template_dir/"
        info "Installed templates"
    fi
}

create_config() {
    step "Creating configuration..."
    
    local config_dir="$HOME/.config/masmide"
    local config_file="$config_dir/config.toml"
    
    mkdir -p "$config_dir"
    
    # Don't overwrite existing config
    if [ -f "$config_file" ]; then
        info "Config already exists at $config_file"
        return
    fi
    
    # Detect linker
    local linker="i686-w64-mingw32-ld"
    check_command "$linker" || linker="x86_64-w64-mingw32-ld"
    
    cat > "$config_file" << EOF
# masmide configuration
# Generated by installer on $(date)

[toolchain]
jwasm_path = "jwasm"
linker_path = "$linker"
wine_path = "wine"
irvine_lib_path = "$LIB_DIR"
irvine_inc_path = "$INC_DIR"

[editor]
tab_size = 4
insert_spaces = true
auto_indent = true
show_line_numbers = true
autosave = true
autosave_interval_secs = 30

[layout]
file_tree_width = 22
output_height = 16

theme_name = "gruvbox"
EOF
    
    info "Created config at $config_file"
}

#───────────────────────────────────────────────────────────────────────────────
# Main
#───────────────────────────────────────────────────────────────────────────────
main() {
    echo
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║   ${CYAN}masmide${NC}${BOLD} - TUI IDE for MASM Development on Linux    ║${NC}"
    echo -e "${BOLD}║              Remote Installer                             ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo
    
    # Check basic requirements
    require_command curl
    require_command tar
    require_command sudo
    
    # Detect system
    local arch=$(detect_arch)
    detect_distro
    info "Detected: $DISTRO_ID ($arch)"
    
    # Get version
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
        if [ -z "$VERSION" ]; then
            error "Failed to get latest version. Try specifying a version: bash -s -- v0.1.0"
        fi
    fi
    info "Version: $VERSION"
    
    echo
    echo "This will install:"
    echo "  • masmide $VERSION to $INSTALL_DIR"
    echo "  • Irvine32 library to $LIB_DIR"
    echo "  • Dependencies (mingw-w64, wine, jwasm)"
    echo
    read -p "Proceed? [Y/n] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Nn]$ ]] && exit 0
    
    echo
    
    # Download release
    download_release "$VERSION" "$arch"
    
    # Install dependencies
    echo
    install_dependencies
    
    # Install masmide
    echo
    install_masmide
    
    # Create config
    echo
    create_config
    
    # Done!
    echo
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║          ${GREEN}Installation Complete!${NC}${BOLD}                         ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo
    echo -e "${BOLD}Quick Start:${NC}"
    echo "  masmide --new myproject   # Create a new project"
    echo "  cd myproject"
    echo "  masmide                   # Open the IDE"
    echo
    echo -e "Run ${CYAN}masmide --help${NC} for more options"
    echo
}

main "$@"
