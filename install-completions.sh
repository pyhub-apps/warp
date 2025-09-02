#!/usr/bin/env bash

# Warp CLI Shell Completions Installer
# Usage: ./install-completions.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BINARY_PATH="${SCRIPT_DIR}/target/release/warp"

# Check if warp binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: warp binary not found at $BINARY_PATH${NC}"
    echo "Please build the project first with: cargo build --release"
    exit 1
fi

echo "🚀 Installing Warp CLI Shell Completions"
echo "========================================="

# Detect current shell
detect_shell() {
    if [ -n "$BASH_VERSION" ]; then
        echo "bash"
    elif [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$FISH_VERSION" ]; then
        echo "fish"
    else
        # Try to detect from SHELL variable
        case "$SHELL" in
            */bash)
                echo "bash"
                ;;
            */zsh)
                echo "zsh"
                ;;
            */fish)
                echo "fish"
                ;;
            *)
                echo "unknown"
                ;;
        esac
    fi
}

CURRENT_SHELL=$(detect_shell)
echo -e "Detected shell: ${GREEN}$CURRENT_SHELL${NC}"

install_bash_completions() {
    echo "Installing Bash completions..."

    # Try different possible locations
    if [ -d "$HOME/.bash_completion.d" ]; then
        COMPLETION_DIR="$HOME/.bash_completion.d"
    elif [ -d "/etc/bash_completion.d" ] && [ -w "/etc/bash_completion.d" ]; then
        COMPLETION_DIR="/etc/bash_completion.d"
    elif [ -d "/usr/local/etc/bash_completion.d" ] && [ -w "/usr/local/etc/bash_completion.d" ]; then
        COMPLETION_DIR="/usr/local/etc/bash_completion.d"
    else
        # Create user directory if it doesn't exist
        COMPLETION_DIR="$HOME/.bash_completion.d"
        mkdir -p "$COMPLETION_DIR"
    fi

    "$BINARY_PATH" completions bash > "$COMPLETION_DIR/warp.bash"

    echo -e "${GREEN}✓ Bash completions installed to $COMPLETION_DIR/warp.bash${NC}"

    # Add to .bashrc if not already present
    if [ -f "$HOME/.bashrc" ]; then
        if ! grep -q "bash_completion.d/warp.bash" "$HOME/.bashrc"; then
            echo "" >> "$HOME/.bashrc"
            echo "# Warp CLI completions" >> "$HOME/.bashrc"
            echo "[ -f $COMPLETION_DIR/warp.bash ] && source $COMPLETION_DIR/warp.bash" >> "$HOME/.bashrc"
            echo -e "${GREEN}✓ Added to ~/.bashrc${NC}"
        fi
    fi

    echo -e "${YELLOW}Note: Restart your shell or run 'source ~/.bashrc' to enable completions${NC}"
}

install_zsh_completions() {
    echo "Installing Zsh completions..."

    # Check for custom completion directory in fpath
    COMPLETION_DIR=""
    if [ -n "$fpath" ]; then
        # Use first writable directory in fpath
        for dir in ${fpath[@]}; do
            if [ -d "$dir" ] && [ -w "$dir" ]; then
                COMPLETION_DIR="$dir"
                break
            fi
        done
    fi

    # Fallback locations
    if [ -z "$COMPLETION_DIR" ]; then
        if [ -d "$HOME/.zsh/completions" ]; then
            COMPLETION_DIR="$HOME/.zsh/completions"
        elif [ -d "/usr/local/share/zsh/site-functions" ] && [ -w "/usr/local/share/zsh/site-functions" ]; then
            COMPLETION_DIR="/usr/local/share/zsh/site-functions"
        else
            # Create user directory
            COMPLETION_DIR="$HOME/.zsh/completions"
            mkdir -p "$COMPLETION_DIR"
        fi
    fi

    "$BINARY_PATH" completions zsh > "$COMPLETION_DIR/_warp"

    echo -e "${GREEN}✓ Zsh completions installed to $COMPLETION_DIR/_warp${NC}"

    # Add to .zshrc if needed
    if [ "$COMPLETION_DIR" = "$HOME/.zsh/completions" ] && [ -f "$HOME/.zshrc" ]; then
        if ! grep -q "fpath.*\.zsh/completions" "$HOME/.zshrc"; then
            echo "" >> "$HOME/.zshrc"
            echo "# Warp CLI completions" >> "$HOME/.zshrc"
            echo "fpath=(~/.zsh/completions \$fpath)" >> "$HOME/.zshrc"
            echo "autoload -Uz compinit && compinit" >> "$HOME/.zshrc"
            echo -e "${GREEN}✓ Added to ~/.zshrc${NC}"
        fi
    fi

    echo -e "${YELLOW}Note: Restart your shell or run 'exec zsh' to enable completions${NC}"
}

install_fish_completions() {
    echo "Installing Fish completions..."

    COMPLETION_DIR="$HOME/.config/fish/completions"
    mkdir -p "$COMPLETION_DIR"

    "$BINARY_PATH" completions fish > "$COMPLETION_DIR/warp.fish"

    echo -e "${GREEN}✓ Fish completions installed to $COMPLETION_DIR/warp.fish${NC}"
    echo -e "${YELLOW}Note: Completions will be available immediately in new Fish shells${NC}"
}

install_powershell_completions() {
    echo "Installing PowerShell completions..."

    "$BINARY_PATH" completions powershell > warp.ps1

    echo -e "${GREEN}✓ PowerShell completions generated to warp.ps1${NC}"
    echo ""
    echo "To install PowerShell completions, add the following to your PowerShell profile:"
    echo -e "${YELLOW}  . $(pwd)/warp.ps1${NC}"
    echo ""
    echo "You can find your PowerShell profile location by running:"
    echo -e "${YELLOW}  echo \$PROFILE${NC}"
}

# Install for all shells or specific shell
if [ "$1" = "--all" ]; then
    echo "Installing completions for all supported shells..."
    install_bash_completions
    echo ""
    install_zsh_completions
    echo ""
    install_fish_completions
    echo ""
    install_powershell_completions
elif [ "$1" = "--bash" ]; then
    install_bash_completions
elif [ "$1" = "--zsh" ]; then
    install_zsh_completions
elif [ "$1" = "--fish" ]; then
    install_fish_completions
elif [ "$1" = "--powershell" ]; then
    install_powershell_completions
else
    # Install for detected shell
    case "$CURRENT_SHELL" in
        bash)
            install_bash_completions
            ;;
        zsh)
            install_zsh_completions
            ;;
        fish)
            install_fish_completions
            ;;
        *)
            echo -e "${RED}Could not detect shell type${NC}"
            echo ""
            echo "Usage: $0 [--all|--bash|--zsh|--fish|--powershell]"
            echo ""
            echo "Options:"
            echo "  --all        Install completions for all supported shells"
            echo "  --bash       Install Bash completions"
            echo "  --zsh        Install Zsh completions"
            echo "  --fish       Install Fish completions"
            echo "  --powershell Generate PowerShell completions"
            exit 1
            ;;
    esac
fi

echo ""
echo -e "${GREEN}✨ Installation complete!${NC}"
echo ""
echo "You can also manually generate completions with:"
echo "  warp completions bash"
echo "  warp completions zsh"
echo "  warp completions fish"
echo "  warp completions powershell"
