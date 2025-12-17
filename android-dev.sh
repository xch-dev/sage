#!/bin/bash
# Wrapper script for Android development that sources the setup script first

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source the Android environment setup script
if [ -f "$SCRIPT_DIR/setup-android-env.sh" ]; then
    source "$SCRIPT_DIR/setup-android-env.sh"
else
    echo "Error: setup-android-env.sh not found at $SCRIPT_DIR/setup-android-env.sh"
    exit 1
fi

# Run the tauri android dev command
pnpm tauri android dev "$@"


