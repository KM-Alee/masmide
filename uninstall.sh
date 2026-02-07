#!/bin/bash
set -e

#===============================================================================
# masmide - Uninstaller
# Removes masmide, bundled JWasm, and Irvine32 library
#===============================================================================

BIN_DIR="/usr/local/bin"
LIB_DIR="/usr/local/lib/irvine"
INC_DIR="/usr/local/include/irvine"
USER_CONFIG_DIR="$HOME/.config/masmide"

#-------------------------------------------------------------------------------
# Colors
#-------------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}[+]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[x]${NC} $1"; exit 1; }
step()    { echo -e "${CYAN}[>]${NC} ${BOLD}$1${NC}"; }

#-------------------------------------------------------------------------------
# Sudo handling
#-------------------------------------------------------------------------------
ensure_sudo() {
    if [ "$EUID" -eq 0 ]; then
        SUDO=""
    else
        SUDO="sudo"
        if ! sudo -v; then
            error "Failed to obtain sudo privileges"
        fi
    fi
}

#-------------------------------------------------------------------------------
# Main
#-------------------------------------------------------------------------------
main() {
    echo
    echo -e "${BOLD}  masmide uninstaller${NC}"
    echo

    echo "  This will remove:"
    echo "    - $BIN_DIR/masmide"
    echo "    - $BIN_DIR/jwasm (bundled assembler)"
    echo "    - $LIB_DIR (Irvine32 libraries)"
    echo "    - $INC_DIR (Irvine32 includes)"
    echo
    echo -e "  ${YELLOW}Optional:${NC}"
    echo "    - $USER_CONFIG_DIR (user configuration)"
    echo

    read -p "  Proceed with uninstallation? [y/N] " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 0

    ensure_sudo

    # Remove binaries
    step "Removing binaries..."
    for bin in masmide jwasm; do
        if [ -f "$BIN_DIR/$bin" ]; then
            $SUDO rm -f "$BIN_DIR/$bin"
            info "Removed $BIN_DIR/$bin"
        else
            warn "$bin not found in $BIN_DIR"
        fi
    done

    # Remove Irvine library
    step "Removing Irvine32 library..."
    for dir in "$LIB_DIR" "$INC_DIR"; do
        if [ -d "$dir" ]; then
            $SUDO rm -rf "$dir"
            info "Removed $dir"
        fi
    done

    # Ask about user config
    echo
    read -p "  Remove user configuration ($USER_CONFIG_DIR)? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -d "$USER_CONFIG_DIR" ]; then
            rm -rf "$USER_CONFIG_DIR"
            info "Removed $USER_CONFIG_DIR"
        else
            warn "Config directory not found"
        fi
    fi

    echo
    echo -e "${GREEN}${BOLD}  Uninstallation complete.${NC}"
    echo
    echo "  Note: System packages (mingw-w64, wine) were not removed."
    echo "  Remove them manually if no longer needed:"
    echo "    Arch:   sudo pacman -Rs mingw-w64-gcc wine"
    echo "    Debian: sudo apt remove mingw-w64 wine64 wine32"
    echo "    Fedora: sudo dnf remove mingw64-gcc wine"
    echo
}

main "$@"
