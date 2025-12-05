# Android Build Setup

This document explains how to build the Sage wallet for Android.

## Prerequisites

1. Latest CMake
    - Install via Homebrew
    - `brew install cmake`
  
2. **Android NDK**: Install via Android Studio SDK Manager or download directly
   - Recommended: NDK r26d or later (r29+ works)
   - The setup script will auto-detect NDK in `~/Library/Android/sdk/ndk/`

3. **Rust Android targets**: Install the required targets

   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```

4. **bindgen-cli**: Required for building `aws-lc-sys`

   ```bash
   cargo install bindgen-cli --locked
   ```

## Building for Android

1. **Source the environment setup script** (this must be done in each terminal session):

   ```bash
   source setup-android-env.sh
   ```

2. **Build the Android app**:

   ```bash
   pnpm tauri android build
   ```

   Or to run in the emulator:

   ```bash
   pnpm tauri android dev
   ```
