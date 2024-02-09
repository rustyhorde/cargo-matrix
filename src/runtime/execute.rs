// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::feature::FeatureMatrix;
use anyhow::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    env::var_os,
    ffi::OsString,
    path::PathBuf,
    process::{Command, Stdio},
};
use yansi::Paint;

lazy_static! {
    static ref CARGO: OsString = var_os("CARGO").unwrap_or_else(|| "cargo".into());
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum TaskKind {
    Build,
    Check,
    Clippy,
    Test,
}

#[allow(dead_code)]
pub(crate) struct Task {
    kind: TaskKind,
    package: String,
    matrix: FeatureMatrix,
    manifest_path: Option<PathBuf>,
    args: Vec<String>,
    dry_run: bool,
}

impl Task {
    pub(crate) fn new(
        kind: TaskKind,
        package: String,
        matrix: FeatureMatrix,
        manifest_path: Option<PathBuf>,
        args: Vec<String>,
        dry_run: bool,
    ) -> Self {
        Self {
            kind,
            package,
            matrix,
            manifest_path,
            args,
            dry_run,
        }
    }

    pub(crate) fn execute(self) -> Result<()> {
        for feature_set in self.matrix {
            match self.kind {
                TaskKind::Build => print!("{}", Paint::cyan("    Building ").bold()),
                TaskKind::Check => print!("{}", Paint::cyan("    Checking ").bold()),
                TaskKind::Clippy => print!("{}", Paint::cyan("      Clippy ").bold()),
                TaskKind::Test => print!("{}", Paint::cyan("     Testing ").bold()),
            }

            println!("package={} features=[{feature_set}]", self.package,);

            let mut cmd = Command::new(CARGO.as_os_str());

            let cmd = match self.kind {
                TaskKind::Build => cmd.arg("build"),
                TaskKind::Check => cmd.arg("check"),
                TaskKind::Clippy => cmd.arg("clippy"),
                TaskKind::Test => cmd.arg("test"),
            };

            let on_success = || {
                println!(
                    "{} {}",
                    Paint::cyan("      Result").bold(),
                    Paint::bright_green("OK")
                );
                println!();
            };

            let cmd = cmd
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .arg("-p")
                .arg(self.package.clone())
                .arg("--no-default-features");

            let cmd = if feature_set.is_empty() {
                cmd
            } else {
                cmd.arg("-F").arg(&feature_set.to_string())
            };

            let cmd = if let Some(manifest_path) = &self.manifest_path {
                cmd.arg("--manifest-path")
                    .arg(format!("{}", manifest_path.display()))
            } else {
                cmd
            };

            let cmd = cmd.args(self.args.clone());

            display_command(cmd);

            let output = cmd.output()?;
            if output.status.success() {
                on_success();
            }
        }

        Ok(())
    }
}

fn display_command(cmd: &Command) {
    let args = cmd.get_args().map(|x| x.to_string_lossy()).join(" ");
    println!(
        "{} {} {}",
        Paint::cyan("     Running").bold(),
        cmd.get_program().to_string_lossy(),
        args
    );
    println!();
}
