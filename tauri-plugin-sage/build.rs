const COMMANDS: &[&str] = &["is_ndef_available", "get_ndef_payloads", "test_tangem"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
