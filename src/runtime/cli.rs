// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use clap::{crate_authors, crate_description, crate_version, Args, Parser, Subcommand};
use getset::Getters;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub(crate) enum Cargo {
    Matrix(MatrixArgs),
}

#[derive(Args, Debug, Getters)]
#[clap(
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
)]
#[command(version, about, long_about = None)]
#[getset(get = "pub(crate)")]
pub(crate) struct MatrixArgs {
    /// Choose the channel you wish to pull your config from
    #[clap(long, short)]
    channel: Option<String>,

    /// Perform a dry run and print output as if all the jobs succeeded.
    #[clap(long)]
    dry_run: bool,

    /// Specify an explict path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,

    /// Specify a specific package to run matrix against
    #[arg(long, short)]
    package: Option<String>,

    /// The supported cargo subcomand to run
    #[command(subcommand)]
    command: CargoSubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CargoSubcommands {
    /// cargo build
    Build(VarArgs),
    /// cargo check
    Check(VarArgs),
    /// cargo clippy
    Clippy(VarArgs),
    /// cargo test
    Test(VarArgs),
}

#[derive(Args, Debug, Getters)]
#[getset(get = "pub(crate)")]

pub(crate) struct VarArgs {
    /// Arguments to pass to the cargo command
    #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}
