#[cfg(debug_assertions)]
fn main() {
    sage_lib::export_bindings();
}

#[cfg(not(debug_assertions))]
fn main() {
    eprintln!("Binding generation is disabled in release builds.");
    std::process::exit(1);
}
