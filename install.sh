#!/bin/sh

set -e

echo "Starting installation..."

error_exit() {
    echo "$1" 1>&2
    exit 1
}

GITHUB_REPO="https://github.com/anggasct/shorty"

if [ -z "$1" ]; then
    echo "No version specified. Fetching the latest version..."
    SHORTY_URL="$GITHUB_REPO/releases/latest/download/"
else
    SHORTY_VERSION="$1"
    echo "Version specified: $SHORTY_VERSION"
    SHORTY_URL="$GITHUB_REPO/releases/download/$SHORTY_VERSION/"
fi

OS_TYPE="$(uname -s)"
case "$OS_TYPE" in
    Linux*)
        echo "Linux detected."
        FILE_NAME="shorty-linux"
        ;;
    Darwin*)
        echo "macOS detected."
        FILE_NAME="shorty-macos"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        echo "Windows detected."
        FILE_NAME="shorty-windows.exe"
        ;;
    *)
        error_exit "Unsupported OS: $OS_TYPE"
        ;;
esac

BINARY_URL="${SHORTY_URL}${FILE_NAME}"

echo "Downloading shorty from $BINARY_URL..."
if command -v curl >/dev/null 2>&1; then
    curl -L "$BINARY_URL" -o /tmp/shorty
elif command -v wget >/dev/null 2>&1; then
    wget "$BINARY_URL" -O /tmp/shorty
else
    error_exit "Neither curl nor wget found. Please install one of them."
fi

if [ $? -ne 0 ]; then
    echo "Failed to download shorty. Please check your internet connection or the URL."
    exit 1
fi

echo "Installing shorty to /usr/local/bin..."
chmod +x /tmp/shorty
if ! sudo mv /tmp/shorty /usr/local/bin/shorty; then
    error_exit "Failed to move shorty to /usr/local/bin. Please ensure you have sufficient permissions."
fi

# Legacy migration: check for old aliases file and let shorty handle migration
if [ -f "$HOME/.shorty_aliases" ]; then
    echo "Found legacy aliases file. Shorty will automatically migrate it on first run."
fi

# Determine shell and config file
if [ -n "$SUDO_USER" ]; then
    USER_SHELL=$(getent passwd "$SUDO_USER" | cut -d: -f7)
else
    USER_SHELL="$SHELL"
fi

SHELL_NAME=$(basename "$USER_SHELL")
case "$SHELL_NAME" in
    zsh)
        CONFIG_FILE="$HOME/.zshrc"
        SOURCE_LINE="source ~/.shorty/aliases"
        ;;
    bash)
        CONFIG_FILE="$HOME/.bashrc"
        SOURCE_LINE="source ~/.shorty/aliases"
        ;;
    fish)
        CONFIG_FILE="$HOME/.config/fish/config.fish"
        SOURCE_LINE="test -f ~/.shorty/aliases; and source ~/.shorty/aliases"
        ;;
    *)
        echo "Unsupported shell: $SHELL_NAME. Please manually add 'source ~/.shorty/aliases' to your shell configuration."
        echo "Installation complete! Run shorty commands to initialize configuration."
        exit 0
        ;;
esac

# Add sourcing to shell config if not already present
if [ ! -f "$CONFIG_FILE" ]; then
    touch "$CONFIG_FILE"
fi

if ! grep -q "source ~/.shorty/aliases\|\.shorty/aliases" "$CONFIG_FILE"; then
    echo "" >> "$CONFIG_FILE"
    echo "# Load aliases from shorty" >> "$CONFIG_FILE"
    echo "$SOURCE_LINE" >> "$CONFIG_FILE"
    echo "Added alias sourcing to $CONFIG_FILE"
else
    echo "$CONFIG_FILE already contains shorty alias sourcing."
fi

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "1. Run 'source $CONFIG_FILE' or restart your terminal"
echo "2. Run 'shorty --help' to see available commands"
echo "3. Run 'shorty add <alias> <command>' to create your first alias"
echo ""
echo "Note: Configuration files and directories will be created automatically in ~/.shorty/ on first use."