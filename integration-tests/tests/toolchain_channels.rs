/// this asserts sync of rust toolchains channels of file_one vs file_two
pub fn assert_equal(
    toolchain_one: &camino::Utf8PathBuf,
    toolchain_two: &camino::Utf8PathBuf,
) -> cargo_near::CliResult {
    let channels = [toolchain_one, toolchain_two]
        .iter()
        .map(|file| get_toolchain_channel(file.as_std_path()))
        .collect::<Result<Vec<_>, color_eyre::Report>>()?;
    assert_eq!(
        channels[0],
        channels[1],
        "no sync of channels of the toolchain `{}` \
        and `{}`",
        toolchain_one.as_str(),
        toolchain_two.as_str(),
    );
    Ok(())
}

/// parses `rust-toolchain.toml` and returns `channel`
fn get_toolchain_channel(toolchain_file: &std::path::Path) -> color_eyre::Result<String> {
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
