#!/bin/bash
set -e

#===============================================================================
# masmide - Installer
# A TUI IDE for MASM development on Linux
#
# Supports: Arch Linux, Ubuntu/Debian, Fedora/RHEL, and generic Linux
# Installs: masmide, JWasm (bundled), MinGW-w64, Wine, Irvine32 library
#
# Run from the extracted release tarball:
#   cd masmide-vX.Y.Z-linux-x86_64
#   sudo ./install.sh
#===============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Installation paths
BIN_DIR="/usr/local/bin"
LIB_DIR="/usr/local/lib/irvine"
INC_DIR="/usr/local/include/irvine"

#-------------------------------------------------------------------------------
# Colors and output helpers
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
error()   { echo -e "${RED}[x]${NC} $1"; exit 1; }
step()    { echo -e "${BLUE}[>]${NC} ${BOLD}$1${NC}"; }
substep() { echo -e "    ${CYAN}-${NC} $1"; }

#-------------------------------------------------------------------------------
# Distro detection
#-------------------------------------------------------------------------------
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO_ID="$ID"
        DISTRO_NAME="$NAME"
    elif [ -f /etc/arch-release ]; then
        DISTRO_ID="arch"
        DISTRO_NAME="Arch Linux"
    elif [ -f /etc/debian_version ]; then
        DISTRO_ID="debian"
        DISTRO_NAME="Debian"
    else
        DISTRO_ID="unknown"
        DISTRO_NAME="Unknown"
    fi

    case "$DISTRO_ID" in
        arch|manjaro|endeavouros|garuda|cachyos)
            DISTRO_FAMILY="arch"
            ;;
        debian|ubuntu|linuxmint|pop|elementary|zorin|kali)
            DISTRO_FAMILY="debian"
            ;;
        fedora|rhel|centos|rocky|alma|nobara)
            DISTRO_FAMILY="fedora"
            ;;
        opensuse*|sles)
            DISTRO_FAMILY="suse"
            ;;
        *)
            DISTRO_FAMILY="unknown"
            ;;
    esac
}

#-------------------------------------------------------------------------------
# Sudo handling
#-------------------------------------------------------------------------------
ensure_sudo() {
    if [ "$EUID" -eq 0 ]; then
        SUDO=""
        return
    fi

    SUDO="sudo"

    echo
    step "This installer requires sudo privileges to:"
    substep "Install system packages (mingw-w64, wine)"
    substep "Install binaries to $BIN_DIR"
    substep "Install Irvine library to $LIB_DIR"
    echo

    if ! sudo -v; then
        error "Failed to obtain sudo privileges"
    fi

    # Keep sudo alive in background
    (while true; do sudo -n true; sleep 50; done) 2>/dev/null &
    SUDO_KEEPALIVE_PID=$!
    trap "kill $SUDO_KEEPALIVE_PID 2>/dev/null" EXIT
}

#-------------------------------------------------------------------------------
# Package installation
#-------------------------------------------------------------------------------
install_packages() {
    step "Installing system dependencies..."

    case "$DISTRO_FAMILY" in
        arch)
            $SUDO pacman -Sy --noconfirm --needed mingw-w64-gcc wine
            ;;
        debian)
            if ! dpkg --print-foreign-architectures 2>/dev/null | grep -q i386; then
                substep "Enabling 32-bit architecture for wine32..."
                $SUDO dpkg --add-architecture i386
            fi
            $SUDO apt-get update -qq
            $SUDO apt-get install -y mingw-w64 wine64 wine32
            ;;
        fedora)
            $SUDO dnf install -y mingw64-gcc wine
            ;;
        suse)
            $SUDO zypper install -y mingw64-cross-gcc wine
            ;;
        *)
            warn "Unknown distribution -- skipping automatic package install."
            warn "Please install these manually:"
            substep "mingw-w64 (cross-compiler / linker)"
            substep "wine      (Windows EXE runner)"
            echo
            ;;
    esac

    info "System dependencies handled"
}

#-------------------------------------------------------------------------------
# Install binaries
#-------------------------------------------------------------------------------
install_binaries() {
    step "Installing binaries to $BIN_DIR..."

    # masmide
    if [ -f "$SCRIPT_DIR/masmide" ]; then
        $SUDO install -m 755 "$SCRIPT_DIR/masmide" "$BIN_DIR/masmide"
        info "Installed masmide"
    else
        error "masmide binary not found in $SCRIPT_DIR"
    fi

    # jwasm (bundled)
    if [ -f "$SCRIPT_DIR/jwasm" ]; then
        $SUDO install -m 755 "$SCRIPT_DIR/jwasm" "$BIN_DIR/jwasm"
        info "Installed jwasm (bundled)"
    else
        warn "jwasm binary not found in tarball -- skipping"
        warn "You may need to install JWasm manually"
    fi
}

#-------------------------------------------------------------------------------
# Install Irvine32 library
#-------------------------------------------------------------------------------
install_irvine() {
    step "Installing Irvine32 library..."

    if [ ! -d "$SCRIPT_DIR/Irvine" ]; then
        warn "Irvine directory not found -- skipping"
        return 1
    fi

    $SUDO mkdir -p "$LIB_DIR" "$INC_DIR"

    substep "Libraries -> $LIB_DIR"
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.lib "$LIB_DIR/" 2>/dev/null || true
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.Lib "$LIB_DIR/" 2>/dev/null || true
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.obj "$LIB_DIR/" 2>/dev/null || true

    substep "Includes  -> $INC_DIR"
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.inc "$INC_DIR/" 2>/dev/null || true

    info "Irvine32 library installed"
}

#-------------------------------------------------------------------------------
# Create user config
#-------------------------------------------------------------------------------
create_config() {
    step "Creating default configuration..."

    local config_dir="$HOME/.config/masmide"
    local config_file="$config_dir/config.toml"

    # Do not overwrite existing config
    if [ -f "$config_file" ]; then
        info "Config already exists at $config_file -- skipping"
        return
    fi

    mkdir -p "$config_dir"

    # Detect linker
    local linker_path="i686-w64-mingw32-ld"
    if ! command -v "$linker_path" &>/dev/null; then
        if command -v x86_64-w64-mingw32-ld &>/dev/null; then
            linker_path="x86_64-w64-mingw32-ld"
        fi
    fi

    cat > "$config_file" << EOF
# masmide configuration
# Generated by installer on $(date)

[toolchain]
jwasm_path = "jwasm"
linker_path = "$linker_path"
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

    # Install templates
    if [ -d "$SCRIPT_DIR/templates" ]; then
        substep "Installing templates..."
        mkdir -p "$config_dir/templates"
        cp -r "$SCRIPT_DIR/templates/"* "$config_dir/templates/"
    fi

    info "Configuration written to $config_file"
}

#-------------------------------------------------------------------------------
# Verify installation
#-------------------------------------------------------------------------------
verify_installation() {
    step "Verifying installation..."

    local ok=true

    if command -v masmide &>/dev/null; then
        substep "masmide: $(masmide --version 2>/dev/null || echo 'installed')"
    else
        warn "masmide not found in PATH"
        ok=false
    fi

    if command -v jwasm &>/dev/null; then
        substep "jwasm: found"
    else
        warn "jwasm not found in PATH"
        ok=false
    fi

    if command -v i686-w64-mingw32-ld &>/dev/null || command -v x86_64-w64-mingw32-ld &>/dev/null; then
        substep "MinGW linker: found"
    else
        warn "MinGW linker not found"
        ok=false
    fi

    if command -v wine &>/dev/null; then
        substep "Wine: $(wine --version 2>/dev/null || echo 'found')"
    else
        warn "Wine not found"
        ok=false
    fi

    if [ -f "$LIB_DIR/Irvine32.lib" ] || [ -f "$LIB_DIR/irvine32.lib" ]; then
        substep "Irvine32: installed"
    else
        warn "Irvine32 library not found"
    fi

    echo
    if [ "$ok" = true ]; then
        info "All components verified"
    else
        warn "Some components may need manual installation"
    fi
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    echo
    echo -e "${BOLD}  masmide installer${NC}"
    echo -e "  TUI IDE for MASM development on Linux"
    echo

    detect_distro
    info "Detected: $DISTRO_NAME ($DISTRO_FAMILY)"

    if [ "$DISTRO_FAMILY" = "unknown" ]; then
        warn "Unsupported distribution. Dependency installation may need manual steps."
        read -p "  Continue anyway? [y/N] " -n 1 -r
        echo
        [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
    fi

    echo
    echo "  This will install:"
    echo "    - masmide + jwasm to $BIN_DIR"
    echo "    - Irvine32 library to $LIB_DIR"
    echo "    - System packages: mingw-w64, wine"
    echo
    read -p "  Proceed? [Y/n] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Nn]$ ]] && exit 0

    echo
    ensure_sudo

    echo
    install_packages

    echo
    install_binaries

    echo
    install_irvine

    echo
    create_config

    echo
    verify_installation

    echo
    echo -e "${GREEN}${BOLD}  Installation complete!${NC}"
    echo
    echo "  Quick start:"
    echo "    masmide --new myproject"
    echo "    cd myproject"
    echo "    masmide"
    echo
    echo "  Keybindings:"
    echo "    F5      Build and run"
    echo "    F6      Build only"
    echo "    Ctrl+S  Save"
    echo "    :q      Quit"
    echo "    F1      Help"
    echo
    echo -e "  Run ${CYAN}masmide --help${NC} for more options"
    echo -e "  Run ${CYAN}./uninstall.sh${NC} to remove"
    echo
}

main "$@"
