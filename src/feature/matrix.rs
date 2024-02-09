// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use super::{Feature, FeatureSet};
use crate::config::Config;
use anyhow::Result;
use cargo_metadata::Package;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    ops::Deref as OpsDeref,
};

#[derive(AsMut, AsRef, Clone, Debug, Default, Deref, DerefMut, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct Matrix(BTreeSet<FeatureSet>);

impl Matrix {
    pub(crate) fn new(package: &Package, config: &Config, channel: &str) -> Result<Self> {
        let deny = config.always_deny(channel)?;
        let skip = config.skip(channel)?;
        let include = config.always_include(channel)?;

        Ok(Self::extract_seed(package, config, channel)?
            .into_iter()
            .powerset()
            .map(FeatureSet::from_iter)
            // Add back the always included features
            .map(|mut set| {
                set.extend(include.clone());
                set
            })
            // Re-check deny in case a custom seed was used
            .filter(|set| set.is_disjoint(&deny))
            // Skip any configured matricies
            .filter(|set| !skip.iter().any(|skip| skip == set))
            .collect())
    }

    /// Reads the package + config and outputs the set of features that should be used to seed the matrix.
    fn extract_seed(package: &Package, config: &Config, channel: &str) -> Result<FeatureSet> {
        Ok(if let Some(seed) = config.seed(channel)? {
            seed.clone()
        } else {
            let implicit_features = Self::find_implicits(package);
            let deny = config.always_deny(channel)?;
            let include = config.always_include(channel)?;

            let mut set: FeatureSet = package
                .features
                .keys()
                .map(Into::into)
                // exclude default feature
                .filter(|feature: &Feature| **feature != "default")
                // exclude implicit features
                .filter(|feature| !implicit_features.contains(feature))
                // exclude deny list because they will all end up denied anyways
                .filter(|package| !deny.iter().contains(package))
                // exclude the include list because it'll be easier to just add them all at once
                .filter(|package| !include.iter().contains(package))
                // exclude hidden features unless explicitly included
                .filter(|feature| {
                    config.include_hidden(channel).unwrap_or_default() || !feature.starts_with("__")
                })
                .collect();

            if config.include_all_optional(channel).unwrap_or_default() {
                set.extend(
                    package
                        .dependencies
                        .iter()
                        .filter(|dependency| dependency.optional)
                        .map(|dependency| {
                            dependency
                                .rename
                                .as_deref()
                                .unwrap_or(&dependency.name)
                                .to_string()
                        })
                        .map(Feature),
                );
            }

            // Add in the specific optional dependencies requested
            set.extend(config.include_optional(channel)?);

            set
        })
    }

    fn find_implicits(package: &Package) -> HashSet<Feature> {
        let mut implicit_features = HashSet::<Feature>::new();
        let mut optional_dep: HashSet<Feature> = HashSet::new();

        for (feature, implied_features) in &package.features {
            for implied_dep in implied_features
                .iter()
                .filter_map(|v| v.strip_prefix("dep:"))
            {
                if implied_features.len() == 1 && implied_dep == feature {
                    // Feature of the shape foo = ["dep:foo"]
                    let _ = implicit_features.insert(feature.clone().into());
                } else {
                    let _ = optional_dep.insert(implied_dep.into());
                }
            }
        }

        // If the dep is used with `dep:` syntax in another feature,
        // it's an explicit feature, because cargo wouldn't generate
        // the implicit feature.
        for x in &optional_dep {
            let _ = implicit_features.remove(x);
        }

        implicit_features
    }
}

impl FromIterator<FeatureSet> for Matrix {
    fn from_iter<T: IntoIterator<Item = FeatureSet>>(iter: T) -> Self {
        Matrix(iter.into_iter().collect())
    }
}

impl IntoIterator for Matrix {
    type Item = FeatureSet;
    type IntoIter = <<Self as OpsDeref>::Target as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
