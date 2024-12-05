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
    *)
        error_exit "Unsupported OS: $OS_TYPE"
        ;;
esac

BINARY_URL="${SHORTY_URL}${FILE_NAME}"

echo "Downloading shorty from $BINARY_URL..."
curl -L "$BINARY_URL" -o /tmp/shorty
if [ $? -ne 0 ]; then
    echo "Failed to download shorty. Please check your internet connection or the URL."
    exit 1
fi

echo "Installing shorty to /usr/local/bin..."
chmod +x /tmp/shorty
if ! sudo mv /tmp/shorty /usr/local/bin/shorty; then
    error_exit "Failed to move shorty to /usr/local/bin. Please ensure you have sufficient permissions."
fi

if [ ! -f "$HOME/.shorty_aliases" ]; then
    touch "$HOME/.shorty_aliases"
    echo "Created ~/.shorty_aliases for storing aliases."
fi

if [ -n "$SUDO_USER" ]; then
    USER_SHELL=$(getent passwd "$SUDO_USER" | cut -d: -f7)
else
    USER_SHELL="$SHELL"
fi

SHELL_NAME=$(basename "$USER_SHELL")
case "$SHELL_NAME" in
    zsh)
        CONFIG_FILE="$HOME/.zshrc"
        ;;
    bash)
        CONFIG_FILE="$HOME/.bashrc"
        ;;
    *)
        error_exit "Unsupported shell: $SHELL_NAME. Please manually add 'source ~/.shorty_aliases' to your shell configuration."
        ;;
esac

if ! grep -q "source ~/.shorty_aliases" "$CONFIG_FILE"; then
    echo "\n# Load aliases from shorty" >> "$CONFIG_FILE"
    echo "source ~/.shorty_aliases" >> "$CONFIG_FILE"
    echo "Added 'source ~/.shorty_aliases' to $CONFIG_FILE"
else
    echo "$CONFIG_FILE already contains the source command."
fi

echo "Installation complete!"
echo "Run 'source $CONFIG_FILE' or restart your terminal to apply changes."
