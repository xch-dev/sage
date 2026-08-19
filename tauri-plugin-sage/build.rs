const COMMANDS: &[&str] = &[
    "is_ndef_available",
    "get_ndef_payloads",
    "set_webview_bounds",
    "snapshot_webview",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
