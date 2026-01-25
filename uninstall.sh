#!/bin/bash
set -e

#═══════════════════════════════════════════════════════════════════════════════
# masmide - Uninstaller
# Removes masmide and optionally JWasm/Irvine library
#═══════════════════════════════════════════════════════════════════════════════

# Installation paths
BIN_DIR="/usr/local/bin"
LIB_DIR="/usr/local/lib/irvine"
INC_DIR="/usr/local/include/irvine"
USER_CONFIG_DIR="$HOME/.config/masmide"

#───────────────────────────────────────────────────────────────────────────────
# Colors
#───────────────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${GREEN}[✓]${NC} $1"; }
warn()    { echo -e "${YELLOW}[!]${NC} $1"; }
error()   { echo -e "${RED}[✗]${NC} $1"; exit 1; }
step()    { echo -e "${CYAN}[→]${NC} ${BOLD}$1${NC}"; }

#───────────────────────────────────────────────────────────────────────────────
# Sudo handling
#───────────────────────────────────────────────────────────────────────────────
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

#───────────────────────────────────────────────────────────────────────────────
# Main
#───────────────────────────────────────────────────────────────────────────────
main() {
    echo
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║              masmide Uninstaller                          ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo

    echo "This will remove:"
    echo "  • masmide binary from $BIN_DIR"
    echo "  • Irvine32 library from $LIB_DIR"
    echo "  • Irvine32 includes from $INC_DIR"
    echo
    echo -e "${YELLOW}Optional:${NC}"
    echo "  • JWasm from $BIN_DIR (if installed by us)"
    echo "  • User configuration from $USER_CONFIG_DIR"
    echo

    read -p "Proceed with uninstallation? [y/N] " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 0

    ensure_sudo

    # Remove masmide binary
    step "Removing masmide..."
    if [ -f "$BIN_DIR/masmide" ]; then
        $SUDO rm -f "$BIN_DIR/masmide"
        info "Removed $BIN_DIR/masmide"
    else
        warn "masmide binary not found"
    fi

    # Remove Irvine library
    step "Removing Irvine32 library..."
    if [ -d "$LIB_DIR" ]; then
        $SUDO rm -rf "$LIB_DIR"
        info "Removed $LIB_DIR"
    else
        warn "Irvine library directory not found"
    fi

    if [ -d "$INC_DIR" ]; then
        $SUDO rm -rf "$INC_DIR"
        info "Removed $INC_DIR"
    else
        warn "Irvine include directory not found"
    fi

    # Ask about JWasm
    echo
    read -p "Remove JWasm from $BIN_DIR? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -f "$BIN_DIR/jwasm" ]; then
            $SUDO rm -f "$BIN_DIR/jwasm"
            info "Removed $BIN_DIR/jwasm"
        else
            warn "jwasm not found in $BIN_DIR"
        fi
    fi

    # Ask about user config
    echo
    read -p "Remove user configuration from $USER_CONFIG_DIR? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -d "$USER_CONFIG_DIR" ]; then
            rm -rf "$USER_CONFIG_DIR"
            info "Removed $USER_CONFIG_DIR"
        else
            warn "User config directory not found"
        fi
    fi

    echo
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║          ${GREEN}Uninstallation Complete!${NC}${BOLD}                       ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo
    echo "Note: System packages (mingw-w64, wine) were not removed."
    echo "Remove them manually if no longer needed:"
    echo
    echo "  Arch:   sudo pacman -Rs mingw-w64-gcc wine"
    echo "  Debian: sudo apt remove mingw-w64 wine"
    echo
}

main "$@"
