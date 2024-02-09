// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;
mod execute;

use self::cli::{Cargo, CargoSubcommands, MatrixArgs};
use crate::{
    config::Config,
    feature::FeatureMatrix,
    runtime::execute::{Task, TaskKind},
};
use anyhow::Result;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::Parser;
use figment::{
    providers::{Format, Json},
    Figment,
};
use std::ffi::OsString;
use yansi::Paint;

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
            let metadata = load_metadata(&matrix_args)?;
            let manifest_path = matrix_args.manifest_path();
            let channel = matrix_args.channel().as_deref().unwrap_or("default");
            let matricies: Vec<(&Package, FeatureMatrix)> = get_workspace_members(&metadata)
                .map(generate_config)
                .filter_map(Result::ok)
                .map(|(package, config)| generate_matrix(package, &config, channel))
                .filter_map(Result::ok)
                .collect();
            println!();
            println!(
                "{} Using channel config '{channel}'",
                Paint::cyan("     Channel").bold()
            );
            println!();

            let (task_kind, varargs) = match matrix_args.command() {
                CargoSubcommands::Build(varargs) => (TaskKind::Build, varargs),
                CargoSubcommands::Check(varargs) => (TaskKind::Check, varargs),
                CargoSubcommands::Clippy(varargs) => (TaskKind::Clippy, varargs),
                CargoSubcommands::Test(varargs) => (TaskKind::Test, varargs),
            };

            let matricies = if let Some(package) = matrix_args.package() {
                matricies
                    .iter()
                    .filter(|(pkg, _)| pkg.name == *package)
                    .cloned()
                    .collect()
            } else {
                matricies
            };

            for (package, matrix) in matricies {
                Task::new(
                    task_kind,
                    package.name.clone(),
                    matrix,
                    manifest_path.clone(),
                    varargs.args().clone(),
                    *matrix_args.dry_run(),
                )
                .execute()?;
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

fn load_metadata(matrix_args: &MatrixArgs) -> Result<Metadata> {
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = matrix_args.manifest_path() {
        let _ = cmd.manifest_path(manifest_path);
    }
    Ok(cmd.exec()?)
}

fn generate_config(package: &Package) -> Result<(&Package, Config)> {
    let figment = if let Some(package_config) = package.metadata.get("cargo-matrix") {
        Figment::from(Config::default())
            .merge(Figment::from(Json::string(&package_config.to_string())))
    } else {
        Figment::from(Config::default())
    };
    Ok((package, Config::from(figment)?))
}

fn generate_matrix<'a>(
    package: &'a Package,
    config: &Config,
    channel: &str,
) -> Result<(&'a Package, FeatureMatrix)> {
    Ok((package, FeatureMatrix::new(package, config, channel)?))
}
