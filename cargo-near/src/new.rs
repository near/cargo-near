use rust_embed::RustEmbed;

use crate::{util, NewCommand};
#[derive(RustEmbed)]
#[folder = "src/templates/"]
struct Assets;
pub fn run(args: NewCommand) -> anyhow::Result<()> {
    args.color.apply();

    util::print_step("Creating new project...");

    std::fs::create_dir_all(&args.project_dir)?;
    for file in Assets::iter() {
        let path = file.as_ref();
        util::print_step(&format!("path: {:?}", path));
        std::fs::create_dir_all(&args.project_dir.join(path).parent().unwrap())?;
        let path = args.project_dir.join(path);
        util::write_file(&path, &Assets::get(&file).unwrap().data)?;
    }

    util::print_success("Created new project...");
    Ok(())
}
