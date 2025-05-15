/// parses `rust-toolchain.toml` and returns `channel`
pub fn get_channel(toolchain_file: &camino::Utf8PathBuf) -> color_eyre::Result<String> {
    let toml_table_str = {
        let bytes = std::fs::read(toolchain_file).map_err(|err| {
            color_eyre::eyre::eyre!("read file, {:?}, err {}", toolchain_file, err)
        })?;
        core::str::from_utf8(&bytes)?.to_owned()
    };
    let toml_table = toml_table_str.parse::<toml::Table>()?;
    let entry = toml_table["toolchain"]["channel"].clone();
    let result = match entry {
        toml::Value::String(channel_string) => channel_string,
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "unexpected variant of toml.toolchain.channel"
            ))
        }
    };
    Ok(result)
}
