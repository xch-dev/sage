# Deep Linking in Sage

Sage supports custom URL scheme deep linking via the `chia-offer://` protocol. When a `chia-offer://` URL is opened, the app will launch (or come to focus if already running) and navigate to the appropriate screen.

## URL Format

```html
chia-offer://<offer_string></offer_string>
```

Where `<offer_string>` is a valid Chia offer string starting with `offer1`.

### Example

```html
chia-offer://offer1qqr83wcuu2rykcmqvpsxvgqqd2h6fv0lt2sn8ntyc6t52p2dxeem089j2x7fua0ygjusmd7wlcdkuk3swhc8d6nga5l447u06ml28d0ldrkn7khmlu063ksltjuz7vxl7uu7zhtt50cmdwv32yfh9r7yx2hq7vylhvmrqmn8vrt7uxje45423vjltcf9ep74nm2jm6kuj8ua3fffandh443zlxdf7f48vuewuk4k0hj4c6z4x8d2yg9zl08s3y2ewpaqna7nfa4agfddd069vpx2glkrvzuuh3xvxa97u00hel344vva6lcrjky2ez53p6yh7uh54rlkxtawmgah0v6v3h36wnw6z3uazgpa5afvmmwelunfzp6y9zpas4ea0hmd8mu30v9t60p7470ntl7djjkrufar4u72yv489hpzx3gknypm8lqzzefu20n36hjz0km5y4wl595u38n8d8a4hnjtmx4lm79la3788yflaq28j5yzhq7cul742jxlcs67f2848k7a60vhkclmaxxqwhxlqu8t6t4kw8kejjmm4nsz9tvj88m87tak3k99efxc7f82kk9s4mu8wz48my300x2t6j8g0ptasnnqqhznpycgvksqph04cd4g72zmwre95sa74dth2h4fpx03fx9pl7t8kmuye7cev4cf0wx7kdqymlz8knj4ej94zma287vtmspkcfgg9fml32229z0l94h5872tjqnf56xmdq3kmdy3xmdysxmd5jxnd5kqv23c6drcurlplmydk366yejl6vfeu99wd47h2fv7u9dv8lee579808p3v8040
```

## Platform-Specific Information

### macOS

#### Registration

The `chia-offer://` URL scheme is automatically registered in the app's `Info.plist` during the build process. The Tauri deep-link plugin handles the `CFBundleURLTypes` entries automatically based on the configuration in `tauri.conf.json`.

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
   open "chia-offer://offer1qqr83wcuu..."
   ```

#### Development Limitations

Deep links do **not** work during development with `pnpm tauri dev` on macOS. The app must be bundled and installed in `/Applications` for deep links to be recognized by the system.

---

### Windows

#### Registration

The URL scheme is registered in the Windows Registry during app installation. The Tauri installer (`.msi` or `.exe`) handles this automatically.

Registry entries are created at:

- `HKEY_CURRENT_USER\Software\Classes\chia-offer`
- Or `HKEY_LOCAL_MACHINE\Software\Classes\chia-offer` (for all users)

#### Testing

1. **Build the app:**

   ```bash
   pnpm tauri build
   ```

2. **Install the app:**

   - Run the generated installer from `src-tauri/target/release/bundle/msi/` or `src-tauri/target/release/bundle/nsis/`

3. **Test the deep link:**

   ```cmd
   start chia-offer://offer1qqr83wcuu...
   ```

   Or open the URL in a web browser.

#### Development Testing

On Windows, you can use `register_all()` in Rust to register the URL scheme during development without installing the app. However, this requires running with elevated permissions.

---

### Linux

#### Registration

On Linux, the URL scheme is registered via a `.desktop` file that includes `MimeType=x-scheme-handler/chia-offer`. This is handled automatically when:

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
   xdg-open "chia-offer://offer1qqr83wcuu..."
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
MimeType=x-scheme-handler/chia-offer;
EOF

# Register the handler
xdg-mime default sage-dev.desktop x-scheme-handler/chia-offer
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

   - Open Safari and navigate to `chia-offer://offer1qqr83wcuu...`
   - Or use the command line on a simulator:

     ```bash
     xcrun simctl openurl booted "chia-offer://offer1qqr83wcuu..."
     ```

#### Development Testing

For iOS development, you can test on the simulator or a physical device connected via Xcode. Deep links work in development builds but require the app to be properly signed.

---

### Android

#### Registration

The URL scheme is automatically registered in the app's `AndroidManifest.xml` during the build process. The Tauri plugin adds the necessary `<intent-filter>` with the `chia-offer` scheme.

The generated manifest includes:

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="chia-offer" />
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
   adb shell am start -a android.intent.action.VIEW -d "chia-offer://offer1qqr83wcuu..."
   ```

#### Development Testing

For Android development, you can test on an emulator or physical device:

```bash
# Start the dev server and build
pnpm tauri android dev

# In another terminal, trigger the deep link
adb shell am start -a android.intent.action.VIEW -d "chia-offer://offer1qqr83wcuu..."
```

---

## Configuration

The deep link configuration is located in `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["chia-offer"]
      },
      "mobile": [
        {
          "scheme": ["chia-offer"],
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
- Check the Windows Registry for the `chia-offer` scheme under `HKEY_CURRENT_USER\Software\Classes`
- Try restarting Windows Explorer

### Deep link not working on Linux

- Ensure you're using an AppImage launcher or installed the `.deb` package
- Verify the MIME type is registered: `xdg-mime query default x-scheme-handler/chia-offer`
- Check that the `.desktop` file exists in `~/.local/share/applications/`

### Deep link not working on iOS

- Verify the app is properly signed
- Check that the Info.plist contains the URL scheme
- Review device logs in Xcode for any errors

### Deep link not working on Android

- Verify the AndroidManifest.xml contains the intent filter
- Check `adb logcat` for any activity resolution errors
- Ensure no other app has registered the same scheme

## References

- [Tauri Deep Linking Plugin Documentation](https://v2.tauri.app/plugin/deep-linking/)
- [Tauri Deep Link Plugin API Reference](https://v2.tauri.app/reference/javascript/deep-link/)
- [Apple URL Scheme Documentation](https://developer.apple.com/documentation/xcode/defining-a-custom-url-scheme-for-your-app)
- [Android Deep Links Documentation](https://developer.android.com/training/app-links/deep-linking)
