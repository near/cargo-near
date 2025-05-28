fn main() {
    // Only show the warning if we're not using cargo-near (cargo-near sets the env var)
    let using_cargo_near = std::env::var("CARGO_NEAR").is_ok();

    if !using_cargo_near {
        println!("");
        println!("\x1b[1;33m>> ⚠️  Please build using `cargo near build` instead << \n\x1b[0m");
        println!("- \x1b[1;32mInstall\x1b[0m cargo NEAR (https://github.com/near/cargo-near)");
        println!("- \x1b[1;34mBuild\x1b[0m your contract with `\x1b[1;37mcargo near build\x1b[0m`");
        println!("- Deploy your contract with `cargo near deploy`");
        println!("");

        // exit with an error code
        std::process::exit(1);
    }
}