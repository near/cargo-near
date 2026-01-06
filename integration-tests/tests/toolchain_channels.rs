/// parses `rust-toolchain.toml` and returns `channel`
pub fn get_channel(toolchain_file: &camino::Utf8PathBuf) -> String {
    let toml_table_str = {
        let bytes = std::fs::read(toolchain_file)
            .map_err(|err| format!("read file, {:?}, err {}", toolchain_file, err))
            .unwrap();
        core::str::from_utf8(&bytes).unwrap().to_owned()
    };
    let toml_table = toml_table_str.parse::<toml::Table>().unwrap();
    let entry = toml_table["toolchain"]["channel"].clone();
    if let toml::Value::String(channel_string) = entry {
        channel_string
    } else {
        panic!("unexpected variant of toml.toolchain.channel");
    }
}
