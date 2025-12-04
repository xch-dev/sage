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

# Add NDK toolchain to PATH
NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
if [ -d "$NDK_BIN" ]; then
    export PATH="$NDK_BIN:$PATH"
    echo "Added NDK toolchain to PATH: $NDK_BIN"
else
    echo "Warning: NDK toolchain bin directory not found: $NDK_BIN"
fi

# Find the highest API level clang for bindgen
# NDK 29 uses versioned clang binaries (e.g., aarch64-linux-android34-clang)
# We'll use API level 34 (Android 14) as default
API_LEVEL=34
SYSROOT="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/sysroot"

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

# Set bindgen to use the NDK's sysroot
# This is critical for aws-lc-sys which uses bindgen to generate bindings
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$SYSROOT"
echo "Configured bindgen sysroot: $SYSROOT"

echo "Android NDK environment configured:"
echo "  ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
echo "  ANDROID_NDK: $ANDROID_NDK"
echo "  NDK_HOME: $NDK_HOME"

