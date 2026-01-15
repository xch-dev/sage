#!/bin/bash
# Setup script for Android NDK environment
# Source this file before building: source setup-android-env.sh

# Detect NDK path
if [ -z "$ANDROID_NDK_HOME" ]; then
    # Try common locations
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        # Use the latest NDK version found
        NDK_PATH=$(ls -td "$HOME/Library/Android/sdk/ndk"/* 2>/dev/null | head -1)
        if [ -n "$NDK_PATH" ]; then
            export ANDROID_NDK_HOME="$NDK_PATH"
        fi
    fi
fi

if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "Error: ANDROID_NDK_HOME not set and could not auto-detect NDK path"
    echo "Please set ANDROID_NDK_HOME to your NDK installation directory"
    return 1
fi

export ANDROID_NDK="$ANDROID_NDK_HOME"
export NDK_HOME="$ANDROID_NDK_HOME"

# Detect host OS for NDK prebuilt path
case "$(uname -s)" in
    Linux*)  HOST_TAG="linux-x86_64" ;;
    Darwin*) HOST_TAG="darwin-x86_64" ;;
    MINGW*|MSYS*|CYGWIN*) HOST_TAG="windows-x86_64" ;;
    *)
        echo "Error: Unsupported host OS: $(uname -s)"
        return 1
        ;;
esac

# Add NDK toolchain to PATH
NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"
if [ -d "$NDK_BIN" ]; then
    export PATH="$NDK_BIN:$PATH"
    echo "Added NDK toolchain to PATH: $NDK_BIN"
else
    echo "Warning: NDK toolchain bin directory not found: $NDK_BIN"
fi

# Auto-detect API level from available clang binaries
# Look for the highest available API level (e.g., aarch64-linux-android35-clang)
API_LEVEL=$(ls "$NDK_BIN"/aarch64-linux-android*-clang 2>/dev/null | \
    sed -n 's/.*android\([0-9]*\)-clang$/\1/p' | \
    sort -n | tail -1)

if [ -z "$API_LEVEL" ]; then
    echo "Warning: Could not auto-detect API level, defaulting to 34"
    API_LEVEL=34
else
    echo "Auto-detected API level: $API_LEVEL"
fi

# Configure CC for all Android targets
# aarch64-linux-android
CLANG_BIN="$NDK_BIN/aarch64-linux-android${API_LEVEL}-clang"
if [ -f "$CLANG_BIN" ]; then
    export CC_aarch64_linux_android="$CLANG_BIN"
    export CXX_aarch64_linux_android="$NDK_BIN/aarch64-linux-android${API_LEVEL}-clang++"
    echo "Configured aarch64-linux-android: $CLANG_BIN"
else
    echo "Warning: Clang binary not found: $CLANG_BIN"
fi

# armv7-linux-androideabi
CLANG_BIN="$NDK_BIN/armv7a-linux-androideabi${API_LEVEL}-clang"
if [ -f "$CLANG_BIN" ]; then
    export CC_armv7_linux_androideabi="$CLANG_BIN"
    export CXX_armv7_linux_androideabi="$NDK_BIN/armv7a-linux-androideabi${API_LEVEL}-clang++"
    echo "Configured armv7-linux-androideabi: $CLANG_BIN"
fi

# i686-linux-android
CLANG_BIN="$NDK_BIN/i686-linux-android${API_LEVEL}-clang"
if [ -f "$CLANG_BIN" ]; then
    export CC_i686_linux_android="$CLANG_BIN"
    export CXX_i686_linux_android="$NDK_BIN/i686-linux-android${API_LEVEL}-clang++"
    echo "Configured i686-linux-android: $CLANG_BIN"
fi

# x86_64-linux-android
CLANG_BIN="$NDK_BIN/x86_64-linux-android${API_LEVEL}-clang"
if [ -f "$CLANG_BIN" ]; then
    export CC_x86_64_linux_android="$CLANG_BIN"
    export CXX_x86_64_linux_android="$NDK_BIN/x86_64-linux-android${API_LEVEL}-clang++"
    echo "Configured x86_64-linux-android: $CLANG_BIN"
fi

# Note: BINDGEN_EXTRA_CLANG_ARGS is configured automatically by build.rs
# during the cargo build process. It handles architecture detection dynamically.

echo "Android NDK environment configured:"
echo "  ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
echo "  ANDROID_NDK: $ANDROID_NDK"
echo "  NDK_HOME: $NDK_HOME"

