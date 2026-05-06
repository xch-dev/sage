use sage_apps::{export_system_bridge_typescript, export_user_bridge_typescript};

fn main() {
    let bridge = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "system".to_string());

    let output = match bridge.as_str() {
        "system" => export_system_bridge_typescript(),
        "user" => export_user_bridge_typescript(),
        other => {
            eprintln!("unknown bridge kind: {other}");
            std::process::exit(1);
        }
    };

    match output {
        Ok(ts) => {
            println!("{ts}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
