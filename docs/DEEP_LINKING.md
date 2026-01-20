# Deep Linking in Sage

Sage supports custom URL scheme deep linking via the `sage:` protocol. When a `sage:` URL is opened, the app will launch (or come to focus if already running) and navigate to the appropriate screen.

## URL Formats

### Offer Links

```
sage:<offer_string>[?fee=<fee_in_mojos>]
```

Where `<offer_string>` is a valid Chia offer string starting with `offer1`.

**Example:**

```
sage:offer1qqr83wcuu2rykcmqvpsxvgqq...
```

### Address Links (Send XCH)

```
sage:<address>?amount=<amount_in_mojos>[&fee=<fee_in_mojos>][&memos=<memo_text>]
```

Opens the send screen with pre-filled values.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `address` | Yes | The destination address (xch1... or txch1...) |
| `amount` | No | Amount to send in mojos (1 XCH = 1,000,000,000,000 mojos) |
| `fee` | No | Transaction fee in mojos |
| `memos` | No | Memo text to attach to the transaction |

**Example:**

```
sage:xch1abc123...?amount=1000000000000&fee=1000000&memos=Payment%20for%20services
```

## Android URL Encoding Requirement

**Important:** On Android, the `&` character in query parameters must be URL-encoded as `%26`. Android's Intent system interprets literal `&` as a command separator, which causes the URL to be truncated at the first `&`.

| Platform | `&` (literal) | `%26` (encoded) |
|----------|---------------|-----------------|
| Android | URL truncated | Works |
| iOS | Works | Works |
| macOS | Works | Works |
| Windows | Works | Works |
| Linux | Works | Works |

**For cross-platform compatibility, always use `%26` instead of `&` in deep link URLs with multiple query parameters.**

**Correct (works everywhere):**

```
sage:xch1abc...?amount=1000000000000%26fee=1000000%26memos=hello
```

**Incorrect (fails on Android):**

```
sage:xch1abc...?amount=1000000000000&fee=1000000&memos=hello
```

The app handles both formats, but Android will truncate URLs with literal `&` before they reach the app.

## Platform-Specific Information

### macOS

#### Registration

The `sage:` URL scheme is automatically registered in the app's `Info.plist` during the build process. The Tauri deep-link plugin handles the `CFBundleURLTypes` entries automatically based on the configuration in `tauri.conf.json`.

#### Testing

1. **Build the app:**

   ```bash
   pnpm tauri build
   ```

2. **Install the app:**

   - Copy `src-tauri/target/release/bundle/macos/Sage.app` to `/Applications`
   - Or open the `.dmg` installer and drag to Applications

3. **Test the deep link:**

   ```bash
   open "sage:offer1qqr83wcuu..."
   ```

#### Development Limitations

Deep links do **not** work during development with `pnpm tauri dev` on macOS. The app must be bundled and installed in `/Applications` for deep links to be recognized by the system.

---

### Windows

#### Registration

The URL scheme is registered in the Windows Registry during app installation. The Tauri installer (`.msi` or `.exe`) handles this automatically.

Registry entries are created at:

- `HKEY_CURRENT_USER\Software\Classes\sage`
- Or `HKEY_LOCAL_MACHINE\Software\Classes\sage` (for all users)

#### Testing

1. **Build the app:**

   ```bash
   pnpm tauri build
   ```

2. **Install the app:**

   - Run the generated installer from `src-tauri/target/release/bundle/msi/` or `src-tauri/target/release/bundle/nsis/`

3. **Test the deep link:**

   ```cmd
   start sage:offer1qqr83wcuu...
   ```

   Or open the URL in a web browser.

#### Development Testing

On Windows, you can use `register_all()` in Rust to register the URL scheme during development without installing the app. However, this requires running with elevated permissions.

---

### Linux

#### Registration

On Linux, the URL scheme is registered via a `.desktop` file that includes `MimeType=x-scheme-handler/sage`. This is handled automatically when:

- Installing the `.deb` package
- Using an AppImage with an AppImage launcher

#### Testing

1. **Build the app:**

   ```bash
   pnpm tauri build
   ```

2. **Install the app:**

   - For `.deb`: `sudo dpkg -i src-tauri/target/release/bundle/deb/sage_*.deb`
   - For AppImage: Use an AppImage launcher like [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher)

3. **Test the deep link:**

   ```bash
   xdg-open "sage:offer1qqr83wcuu..."
   ```

#### Development Testing

During development, you can manually create a `.desktop` file or use `xdg-mime` to register the scheme handler:

```bash
# Create a desktop entry (replace paths appropriately)
cat > ~/.local/share/applications/sage-dev.desktop << EOF
[Desktop Entry]
Name=Sage (Dev)
Exec=/path/to/sage %u
Type=Application
MimeType=x-scheme-handler/sage;
EOF

# Register the handler
xdg-mime default sage-dev.desktop x-scheme-handler/sage
```

---

### iOS

#### Registration

The URL scheme is automatically configured in the app's `Info.plist` during the build process. The Tauri plugin generates the necessary `CFBundleURLTypes` entries.

#### Testing

1. **Build for iOS:**

   ```bash
   pnpm tauri ios build
   ```

2. **Install on device/simulator:**

   - Use Xcode to install on a physical device or simulator
   - Or use TestFlight for distribution

3. **Test the deep link:**

   - Open Safari and navigate to `sage:offer1qqr83wcuu...`
   - Or use the command line on a simulator:

     ```bash
     xcrun simctl openurl booted "sage:offer1qqr83wcuu..."
     ```

#### Development Testing

For iOS development, you can test on the simulator or a physical device connected via Xcode. Deep links work in development builds but require the app to be properly signed.

---

### Android

#### Registration

The URL scheme is automatically registered in the app's `AndroidManifest.xml` during the build process. The Tauri plugin adds the necessary `<intent-filter>` with the `sage` scheme.

The generated manifest includes:

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="sage" />
</intent-filter>
```

#### Testing

1. **Build for Android:**

   ```bash
   pnpm tauri android build
   ```

2. **Install on device/emulator:**

   ```bash
   adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
   ```

3. **Test the deep link:**

   ```bash
   # Offer link
   adb shell am start -a android.intent.action.VIEW -d "sage:offer1qqr83wcuu..."

   # Address link with parameters (note: use %26 for &)
   adb shell am start -a android.intent.action.VIEW -d "sage:xch1abc...?amount=1000000000000%26fee=1000000%26memos=hello"
   ```

#### Development Testing

For Android development, you can test on an emulator or physical device:

```bash
# Start the dev server and build
pnpm tauri android dev

# In another terminal, trigger the deep link
adb shell am start -a android.intent.action.VIEW -d "sage:offer1qqr83wcuu..."
```

#### URL Encoding Note

When testing address links with multiple query parameters, remember to use `%26` instead of `&`. See [Android URL Encoding Requirement](#android-url-encoding-requirement) for details.

---

## Configuration

The deep link configuration is located in `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["sage"]
      },
      "mobile": [
        {
          "scheme": ["sage"],
          "appLink": false
        }
      ]
    }
  }
}
```

- **desktop.schemes**: List of URL schemes for desktop platforms (macOS, Windows, Linux)
- **mobile**: Configuration for mobile platforms (iOS, Android)
  - **scheme**: List of URL schemes
  - **appLink**: Set to `false` for custom schemes (no domain verification required)

## Permissions

The following capabilities are required:

### Desktop (`src-tauri/capabilities/desktop.json`)

```json
{
  "permissions": ["deep-link:default"]
}
```

### Mobile (`src-tauri/capabilities/mobile.json`)

```json
{
  "permissions": ["deep-link:default"]
}
```

## Troubleshooting

### Deep link not working on macOS

- Ensure the app is installed in `/Applications`
- Verify the app was built with `pnpm tauri build`, not running in dev mode
- Check Console.app for any launch services errors

### Deep link not working on Windows

- Verify the app was installed via the MSI or NSIS installer
- Check the Windows Registry for the `sage` scheme under `HKEY_CURRENT_USER\Software\Classes`
- Try restarting Windows Explorer

### Deep link not working on Linux

- Ensure you're using an AppImage launcher or installed the `.deb` package
- Verify the MIME type is registered: `xdg-mime query default x-scheme-handler/sage`
- Check that the `.desktop` file exists in `~/.local/share/applications/`

### Deep link not working on iOS

- Verify the app is properly signed
- Check that the Info.plist contains the URL scheme
- Review device logs in Xcode for any errors

### Deep link not working on Android

- Verify the AndroidManifest.xml contains the intent filter
- Check `adb logcat` for any activity resolution errors
- Ensure no other app has registered the same scheme

### Query parameters missing on Android

If only the address is populated but amount, fee, or memos are missing, the URL likely contains literal `&` characters. Android's Intent system truncates URLs at the first `&`. Use `%26` instead:

```
# Wrong - parameters after first & will be lost
sage:xch1...?amount=100&fee=100&memos=test

# Correct - all parameters will be received
sage:xch1...?amount=100%26fee=100%26memos=test
```

## References

- [Tauri Deep Linking Plugin Documentation](https://v2.tauri.app/plugin/deep-linking/)
- [Tauri Deep Link Plugin API Reference](https://v2.tauri.app/reference/javascript/deep-link/)
- [Apple URL Scheme Documentation](https://developer.apple.com/documentation/xcode/defining-a-custom-url-scheme-for-your-app)
- [Android Deep Links Documentation](https://developer.android.com/training/app-links/deep-linking)
