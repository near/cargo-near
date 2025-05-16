use super::section_name;

impl super::AppliedReproducibleBuild {
    pub fn validate(&self) -> eyre::Result<()> {
        self.validate_image()?;
        self.validate_image_digest()?;
        self.validate_container_build_command()?;
        self.validate_if_unknown_keys_present()?;
        self.validate_repository()?;

        Ok(())
    }

    fn validate_image(&self) -> eyre::Result<()> {
        if self
            .image
            .chars()
            .any(|c| !c.is_ascii() || c.is_ascii_control() || c.is_ascii_whitespace())
        {
            let section_name = section_name(self.selected_variant.as_ref());
            return Err(eyre::eyre!(
                "Malformed `{}` in Cargo.toml: `{}`\n{}",
                section_name,
                self.image,
                "`image`: string contains invalid characters",
            ));
        }
        Ok(())
    }
    fn validate_image_digest(&self) -> eyre::Result<()> {
        if self
            .image_digest
            .chars()
            .any(|c| !c.is_ascii() || c.is_ascii_control() || c.is_ascii_whitespace())
        {
            let section_name = section_name(self.selected_variant.as_ref());
            return Err(eyre::eyre!(
                "Malformed `{}` in Cargo.toml: `{}`\n{}",
                section_name,
                self.image_digest,
                "`image_digest`: string contains invalid characters",
            ));
        }
        Ok(())
    }

    fn validate_container_build_command(&self) -> eyre::Result<()> {
        let build_command = self.container_build_command.clone().ok_or(eyre::eyre!(
        "`container_build_command` field not found! It is required since 0.13.0 version of `cargo-near`"
    ))?;
        let is_cargo_near = {
            Some("cargo") == build_command.first().map(AsRef::as_ref)
                && Some("near") == build_command.get(1).map(AsRef::as_ref)
        };

        let section_name = section_name(self.selected_variant.as_ref());

        for command_token in build_command {
            if command_token
                .chars()
                .any(|c| !c.is_ascii() || c.is_ascii_control())
            {
                return Err(eyre::eyre!(
                    "Malformed `{}` in Cargo.toml: `{}`\n{}",
                    section_name,
                    command_token,
                    "`container_build_command`: string token contains invalid characters",
                ));
            }
            // for versions of cargo-near inside of container <0.13
            // versions >=0.13 require `--locked` flag instead, but this isn't validated
            if is_cargo_near && command_token == "--no-locked" {
                return Err(eyre::eyre!(
                "Malformed `{}` in Cargo.toml:\n{}",
                section_name,
                "`container_build_command`: `--no-locked` forbidden for `cargo near` build command",
            ));
            }
            if is_cargo_near && command_token == "--manifest-path" {
                return Err(eyre::eyre!(
                    "Malformed `{}` in Cargo.toml:\n{}\n{}",
                    section_name,
                    "`container_build_command`: `--manifest-path` isn't allowed to be specified \
                    in manifest itself.",
                    "`--manifest-path ./Cargo.toml` is implied in all such cases",
                ));
            }
        }
        Ok(())
    }

    fn validate_if_unknown_keys_present(&self) -> eyre::Result<()> {
        if !self.unknown_keys.is_empty() {
            let keys = self
                .unknown_keys
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>();

            let section_name = section_name(self.selected_variant.as_ref());
            return Err(eyre::eyre!(
                "Malformed `{}` in Cargo.toml, contains unknown keys: `{}`",
                section_name,
                keys.join(",")
            ));
        }
        Ok(())
    }

    fn validate_repository(&self) -> eyre::Result<()> {
        if let Some(ref repository) = self.repository {
            if repository.scheme() != "https" {
                Err(eyre::eyre!(
                    "{}: {}\n{}",
                    "Malformed NEP330 metadata in Cargo.toml:",
                    repository,
                    "`[package.repository]`: only `https` scheme is supported at the moment",
                ))
            } else {
                Ok(())
            }
        } else {
            Err(eyre::eyre!(
                "{}: \n{}",
                "Malformed NEP330 metadata in Cargo.toml",
                "`[package.repository]`: should not be empty",
            ))
        }
    }
}
