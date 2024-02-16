// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod cli;
mod execute;

use self::cli::{Cargo, CargoSubcommands};
use crate::{
    config::Config,
    feature::FeatureMatrix,
    runtime::execute::{Task, TaskKind, TaskResult},
};
use anyhow::{anyhow, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::Parser;
use figment::{
    providers::{Format, Json},
    Figment,
};
use std::{ffi::OsString, path::PathBuf};
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
            // Grab the manifest path from the command line, if supplied
            let manifest_path = matrix_args.manifest_path();
            // Read the cargo metadata
            let metadata = load_metadata(manifest_path)?;
            // Determine the channel, default is 'default'
            let channel = matrix_args.channel().as_deref().unwrap_or("default");
            // Generate the feature set matricies for every package in the workspace
            let matricies: Vec<(&Package, FeatureMatrix)> = get_workspace_members(&metadata)
                .map(generate_config)
                .filter_map(Result::ok)
                .map(|(package, config)| generate_matrix(package, &config, channel))
                .filter_map(Result::ok)
                .collect();
            // Output some stuff
            println!();
            println!(
                "{} Using channel config '{channel}'",
                Paint::cyan("     Channel").bold()
            );
            println!();

            // Determine the base command we are running
            let (task_kind, varargs) = match matrix_args.command() {
                CargoSubcommands::Build(varargs) => (TaskKind::Build, varargs),
                CargoSubcommands::Check(varargs) => (TaskKind::Check, varargs),
                CargoSubcommands::Clippy(varargs) => (TaskKind::Clippy, varargs),
                CargoSubcommands::LlvmCov(varargs) => (TaskKind::LlvmCov, varargs),
                CargoSubcommands::Test(varargs) => (TaskKind::Test, varargs),
            };

            // Filter the matricies if a specific package was specified at the command line
            let matricies = if let Some(package) = matrix_args.package() {
                matricies
                    .iter()
                    .filter(|(pkg, _)| pkg.name == *package)
                    .cloned()
                    .collect()
            } else {
                let num_chunks = matrix_args.num_chunks();
                let chunk = matrix_args.chunk();
                if *chunk == 0 {
                    return Err(anyhow!("chunk argument cannot be 0"));
                }
                if *num_chunks == 0 {
                    return Err(anyhow!("num_chunks argument cannot be 0"));
                }
                if chunk > num_chunks {
                    return Err(anyhow!("chunk must be less than or equal to num_chunks"));
                }

                let mut chunk_size = matricies.len() / num_chunks;
                if matricies.len() % num_chunks != 0 {
                    chunk_size += 1;
                }
                let Some(matrix_chunk) = matricies.chunks(chunk_size).nth(chunk - 1) else {
                    println!(
                        "Chunk is empty (did you ask for more chunks than there are packages?"
                    );
                    return Ok(());
                };
                if *num_chunks != 1 {
                    let len = matrix_chunk.len();
                    let packages: String = matrix_chunk
                        .iter()
                        .flat_map(|(p, _)| [&p.name, ","])
                        .collect();
                    let packages = packages.trim_end_matches(',');
                    print!("{}", Paint::cyan("    Chunking ").bold());
                    println!(
                        "Running on chunk {chunk} out of {num_chunks} ({len} package(s): {packages})"
                    );
                }
                matrix_chunk.to_vec()
            };

            // Execute the task against the matricies
            for (package, matrix) in matricies {
                let task_result = Task::new(
                    task_kind,
                    package.name.clone(),
                    matrix,
                    manifest_path.clone(),
                    varargs.args().clone(),
                    *matrix_args.dry_run(),
                )
                .execute()?;
                match task_result {
                    TaskResult::Success => continue,
                    TaskResult::Fail(code) => return Err(anyhow!("task failed: {}", code)),
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

fn load_metadata(manifest_path: &Option<PathBuf>) -> Result<Metadata> {
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
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
