// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;

use crate::{config::Config, feature::FeatureMatrix};

use self::cli::Cargo;
use anyhow::Result;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::Parser;
use figment::{
    providers::{Format, Json},
    Figment,
};
use std::ffi::OsString;

pub(crate) fn run<I, T>(args: Option<I>) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Parse the command line
    let cli = if let Some(args) = args {
        Cargo::try_parse_from(args)?
    } else {
        Cargo::try_parse()?
    };

    match cli {
        Cargo::Matrix(matrix_args) => {
            let mut cmd = MetadataCommand::new();
            if let Some(manifest_path) = matrix_args.manifest_path() {
                let _ = cmd.manifest_path(manifest_path);
            }
            let metadata = cmd.exec()?;

            for package in get_workspace_members(&metadata) {
                let figment = if let Some(package_config) = package.metadata.get("cargo-matrix") {
                    Figment::from(&Config::default())
                        .merge(Figment::from(Json::string(&package_config.to_string())))
                } else {
                    Figment::from(&Config::default())
                };
                let config = Config::from(figment)?;
                let channel = matrix_args.channel().as_deref().unwrap_or("default");
                eprintln!("channel: {channel}");
                let matrix = FeatureMatrix::new(package, &config, channel)?;
                for set in matrix.iter() {
                    eprintln!("{} {set}", package.name);
                }
            }
        }
    }

    Ok(())
}

/// Gets a list of packages that are members of the workspace
fn get_workspace_members(metadata: &Metadata) -> impl Iterator<Item = &Package> + '_ {
    metadata
        .packages
        .iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
}
