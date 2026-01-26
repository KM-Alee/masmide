#!/bin/bash
set -e

#═══════════════════════════════════════════════════════════════════════════════
# masmide - Comprehensive Installer
# A TUI IDE for MASM development on Linux
#
# Supports: Arch Linux, Ubuntu/Debian
# Installs: masmide, JWasm, MinGW-w64, Wine, Irvine32 library
#═══════════════════════════════════════════════════════════════════════════════

VERSION="0.1.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Installation paths (system-wide)
BIN_DIR="/usr/local/bin"
LIB_DIR="/usr/local/lib/irvine"
INC_DIR="/usr/local/include/irvine"
CONFIG_DIR="/etc/masmide"

# JWasm source (will build from source if not available)
JWASM_REPO="https://github.com/JWasm/JWasm.git"
JWASM_BUILD_DIR="/tmp/jwasm-build-$$"

#───────────────────────────────────────────────────────────────────────────────
# Colors and output helpers
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
error()   { echo -e "${RED}[✗]${NC} $1"; exit 1; }
step()    { echo -e "${BLUE}[→]${NC} ${BOLD}$1${NC}"; }
substep() { echo -e "    ${CYAN}•${NC} $1"; }

#───────────────────────────────────────────────────────────────────────────────
# Distro detection
#───────────────────────────────────────────────────────────────────────────────
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

    # Normalize distro ID
    case "$DISTRO_ID" in
        arch|manjaro|endeavouros|garuda)
            DISTRO_FAMILY="arch"
            PKG_MANAGER="pacman"
            ;;
        debian|ubuntu|linuxmint|pop|elementary|zorin|kali)
            DISTRO_FAMILY="debian"
            PKG_MANAGER="apt"
            ;;
        *)
            DISTRO_FAMILY="unknown"
            PKG_MANAGER="unknown"
            ;;
    esac
}

#───────────────────────────────────────────────────────────────────────────────
# Sudo handling
#───────────────────────────────────────────────────────────────────────────────
ensure_sudo() {
    if [ "$EUID" -eq 0 ]; then
        SUDO=""
        return
    fi

    SUDO="sudo"
    
    echo
    step "This installer requires sudo privileges to:"
    substep "Install system packages (mingw-w64, wine)"
    substep "Install masmide to /usr/local/bin"
    substep "Install Irvine library to /usr/local/lib"
    echo

    # Request sudo and keep it alive
    if ! sudo -v; then
        error "Failed to obtain sudo privileges"
    fi

    # Keep sudo alive in background
    (while true; do sudo -n true; sleep 50; done) 2>/dev/null &
    SUDO_KEEPALIVE_PID=$!
    trap "kill $SUDO_KEEPALIVE_PID 2>/dev/null" EXIT
}

#───────────────────────────────────────────────────────────────────────────────
# Package installation
#───────────────────────────────────────────────────────────────────────────────
install_packages_arch() {
    step "Installing packages via pacman..."
    
    # Update package database
    $SUDO pacman -Sy --noconfirm

    # Core packages
    local packages="mingw-w64-gcc wine base-devel git"
    
    for pkg in $packages; do
        if ! pacman -Qi "$pkg" &>/dev/null; then
            substep "Installing $pkg..."
            $SUDO pacman -S --noconfirm --needed "$pkg"
        else
            substep "$pkg already installed"
        fi
    done
    
    # Check for AUR helper and jwasm
    if command -v jwasm &>/dev/null; then
        substep "jwasm already installed"
        JWASM_INSTALLED=true
    elif command -v yay &>/dev/null; then
        substep "Installing jwasm from AUR via yay..."
        yay -S --noconfirm jwasm && JWASM_INSTALLED=true || JWASM_INSTALLED=false
    elif command -v paru &>/dev/null; then
        substep "Installing jwasm from AUR via paru..."
        paru -S --noconfirm jwasm && JWASM_INSTALLED=true || JWASM_INSTALLED=false
    else
        JWASM_INSTALLED=false
    fi
}

install_packages_debian() {
    step "Installing packages via apt..."
    
    # Update package database
    $SUDO apt-get update -qq

    # Core packages - use i686 for 32-bit Windows compatibility
    local packages="mingw-w64 wine64 wine32 build-essential git"
    
    # Check if 32-bit architecture is enabled (needed for wine32)
    if ! dpkg --print-foreign-architectures | grep -q i386; then
        substep "Enabling 32-bit architecture for wine32..."
        $SUDO dpkg --add-architecture i386
        $SUDO apt-get update -qq
    fi
    
    for pkg in $packages; do
        if ! dpkg -l "$pkg" 2>/dev/null | grep -q "^ii"; then
            substep "Installing $pkg..."
            $SUDO apt-get install -y "$pkg"
        else
            substep "$pkg already installed"
        fi
    done
    
    JWASM_INSTALLED=false
    if command -v jwasm &>/dev/null; then
        substep "jwasm already installed"
        JWASM_INSTALLED=true
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# JWasm build from source
#───────────────────────────────────────────────────────────────────────────────
build_jwasm() {
    if [ "$JWASM_INSTALLED" = true ]; then
        return 0
    fi

    step "Building JWasm from source..."
    
    # Clean up any previous build
    rm -rf "$JWASM_BUILD_DIR"
    mkdir -p "$JWASM_BUILD_DIR"
    
    substep "Cloning JWasm repository..."
    git clone --depth 1 "$JWASM_REPO" "$JWASM_BUILD_DIR" 2>/dev/null
    
    substep "Compiling JWasm..."
    cd "$JWASM_BUILD_DIR"
    
    # JWasm uses a simple Makefile
    if [ -f "GccUnix.mak" ]; then
        make -f GccUnix.mak
        JWASM_BIN="$JWASM_BUILD_DIR/build/GccUnixR/jwasm"
    elif [ -f "Makefile" ]; then
        make
        JWASM_BIN="$JWASM_BUILD_DIR/jwasm"
    else
        error "Could not find JWasm makefile"
    fi
    
    if [ ! -f "$JWASM_BIN" ]; then
        error "JWasm build failed - binary not found"
    fi
    
    substep "Installing JWasm to $BIN_DIR..."
    $SUDO cp "$JWASM_BIN" "$BIN_DIR/jwasm"
    $SUDO chmod +x "$BIN_DIR/jwasm"
    
    # Cleanup
    cd "$SCRIPT_DIR"
    rm -rf "$JWASM_BUILD_DIR"
    
    info "JWasm installed successfully"
}

#───────────────────────────────────────────────────────────────────────────────
# Rust/Cargo check and install
#───────────────────────────────────────────────────────────────────────────────
ensure_rust() {
    if command -v cargo &>/dev/null; then
        info "Rust/Cargo found: $(cargo --version)"
        return 0
    fi

    step "Rust not found. Installing via rustup..."
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    
    if ! command -v cargo &>/dev/null; then
        error "Failed to install Rust. Please install manually from https://rustup.rs/"
    fi
    
    info "Rust installed successfully"
}

#───────────────────────────────────────────────────────────────────────────────
# Build and install masmide
#───────────────────────────────────────────────────────────────────────────────
build_masmide() {
    step "Building masmide..."
    
    cd "$SCRIPT_DIR"
    cargo build --release 2>&1 | while read -r line; do
        if [[ "$line" == *"Compiling"* ]]; then
            substep "$line"
        fi
    done
    
    if [ ! -f "target/release/masmide" ]; then
        error "Build failed - binary not found"
    fi
    
    info "Build successful"
}

install_masmide() {
    step "Installing masmide..."
    
    substep "Installing binary to $BIN_DIR..."
    $SUDO cp target/release/masmide "$BIN_DIR/"
    $SUDO chmod +x "$BIN_DIR/masmide"
    
    info "masmide installed to $BIN_DIR/masmide"
}

#───────────────────────────────────────────────────────────────────────────────
# Install Irvine32 library
#───────────────────────────────────────────────────────────────────────────────
install_irvine() {
    step "Installing Irvine32 library..."
    
    if [ ! -d "$SCRIPT_DIR/Irvine" ]; then
        warn "Irvine directory not found - skipping"
        return 1
    fi
    
    # Create directories
    $SUDO mkdir -p "$LIB_DIR" "$INC_DIR"
    
    # Copy library files
    substep "Installing libraries to $LIB_DIR..."
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.lib "$LIB_DIR/" 2>/dev/null || true
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.Lib "$LIB_DIR/" 2>/dev/null || true
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.obj "$LIB_DIR/" 2>/dev/null || true
    
    # Copy include files
    substep "Installing includes to $INC_DIR..."
    $SUDO cp -f "$SCRIPT_DIR"/Irvine/*.inc "$INC_DIR/" 2>/dev/null || true
    
    info "Irvine32 library installed"
}

#───────────────────────────────────────────────────────────────────────────────
# Create global config
#───────────────────────────────────────────────────────────────────────────────
create_config() {
    step "Creating default configuration..."
    
    local user_config_dir="$HOME/.config/masmide"
    mkdir -p "$user_config_dir"
    
    # Detect linker path
    local linker_path="i686-w64-mingw32-ld"
    if ! command -v "$linker_path" &>/dev/null; then
        if command -v x86_64-w64-mingw32-ld &>/dev/null; then
            linker_path="x86_64-w64-mingw32-ld"
        fi
    fi
    
    # Create config file
    cat > "$user_config_dir/config.toml" << EOF
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
        mkdir -p "$user_config_dir/templates"
        cp -r "$SCRIPT_DIR/templates/"* "$user_config_dir/templates/"
    fi
    
    info "Configuration created at $user_config_dir/config.toml"
}

#───────────────────────────────────────────────────────────────────────────────
# Verify installation
#───────────────────────────────────────────────────────────────────────────────
verify_installation() {
    step "Verifying installation..."
    
    local all_ok=true
    
    if command -v masmide &>/dev/null; then
        substep "masmide: $(masmide --version 2>/dev/null || echo 'OK')"
    else
        warn "masmide not found in PATH"
        all_ok=false
    fi
    
    if command -v jwasm &>/dev/null; then
        substep "jwasm: $(jwasm -? 2>&1 | head -1 || echo 'OK')"
    else
        warn "jwasm not found"
        all_ok=false
    fi
    
    if command -v i686-w64-mingw32-ld &>/dev/null || command -v x86_64-w64-mingw32-ld &>/dev/null; then
        substep "MinGW linker: OK"
    else
        warn "MinGW linker not found"
        all_ok=false
    fi
    
    if command -v wine &>/dev/null; then
        substep "Wine: $(wine --version 2>/dev/null || echo 'OK')"
    else
        warn "Wine not found"
        all_ok=false
    fi
    
    if [ -f "$LIB_DIR/Irvine32.lib" ] || [ -f "$LIB_DIR/irvine32.lib" ]; then
        substep "Irvine32 library: OK"
    else
        warn "Irvine32 library not found"
    fi
    
    if [ "$all_ok" = true ]; then
        info "All components verified successfully!"
        return 0
    else
        warn "Some components may need manual configuration"
        return 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Main
#───────────────────────────────────────────────────────────────────────────────
main() {
    clear
    echo
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║                                                           ║${NC}"
    echo -e "${BOLD}║   ${CYAN}masmide${NC}${BOLD} - TUI IDE for MASM Development on Linux         ║${NC}"
    echo -e "${BOLD}║                     Installer v${VERSION}                      ║${NC}"
    echo -e "${BOLD}║                                                           ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo

    # Detect distro
    detect_distro
    info "Detected: $DISTRO_NAME ($DISTRO_FAMILY)"
    
    if [ "$DISTRO_FAMILY" = "unknown" ]; then
        warn "Unsupported distribution. Manual dependency installation may be required."
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
    fi
    
    echo
    echo "This will install:"
    echo "  • masmide IDE to $BIN_DIR"
    echo "  • JWasm assembler (if not present)"
    echo "  • MinGW-w64 linker"
    echo "  • Wine for running executables"
    echo "  • Irvine32 library to $LIB_DIR"
    echo
    read -p "Proceed with installation? [Y/n] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Nn]$ ]] && exit 0
    
    echo
    
    # Get sudo access
    ensure_sudo
    
    # Install system packages
    echo
    case "$DISTRO_FAMILY" in
        arch)   install_packages_arch ;;
        debian) install_packages_debian ;;
    esac
    
    # Build JWasm if needed
    echo
    build_jwasm
    
    # Ensure Rust is available
    echo
    ensure_rust
    
    # Build masmide
    echo
    build_masmide
    
    # Install everything
    echo
    install_masmide
    
    echo
    install_irvine
    
    echo
    create_config
    
    # Verify
    echo
    verify_installation
    
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
    echo -e "${BOLD}Keybindings:${NC}"
    echo "  F5        Build and run"
    echo "  F6        Build only"
    echo "  Ctrl+S    Save"
    echo "  :q        Quit"
    echo "  F1        Help"
    echo
    echo -e "Run ${CYAN}masmide --help${NC} for more options"
    echo -e "Run ${CYAN}./uninstall.sh${NC} to remove masmide"
    echo
}

main "$@"
