fn main() {
    if let Err(err) = sage_apps::generate_docs() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
