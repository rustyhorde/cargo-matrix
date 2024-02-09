// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use super::Feature;
use cargo_metadata::Package;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    fmt::{Display, Formatter},
    ops::Deref as OpsDeref,
};

#[derive(
    Clone,
    Debug,
    Default,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Deref,
    DerefMut,
    AsRef,
    AsMut,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub(crate) struct Set<'f>(BTreeSet<Feature<'f>>);

impl<'f> Set<'f> {
    #[allow(dead_code)]
    pub(crate) fn add_transitive_features(&mut self, package: &'f Package) {
        let raw_features = &package.features;
        let transitive = self
            .iter()
            .filter_map(|feature| {
                raw_features.get(feature.as_ref()).map(|transitives| {
                    transitives
                        .iter()
                        .filter(|transitive| !transitive.starts_with("dep:"))
                        .map(AsRef::as_ref)
                })
            })
            .flatten()
            .map(Cow::Borrowed)
            .map(Feature)
            .collect_vec();
        self.extend(transitive);
    }
}

impl Display for Set<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.iter();
        if let Some(feature) = iter.next() {
            Display::fmt(feature, f)?;
        }
        for feature in iter {
            write!(f, ",{feature}")?;
        }
        Ok(())
    }
}

impl<'f> FromIterator<Feature<'f>> for Set<'f> {
    fn from_iter<T: IntoIterator<Item = Feature<'f>>>(iter: T) -> Self {
        Set(iter.into_iter().collect())
    }
}

impl<'f> IntoIterator for Set<'f> {
    type Item = Feature<'f>;
    type IntoIter = <<Self as OpsDeref>::Target as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
