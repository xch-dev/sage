fn main() {
    let path = dirs::data_dir()
        .expect("could not find data directory")
        .join("com.rigidnetwork.sage");

    println!("Data directory: {path:?}");
}
