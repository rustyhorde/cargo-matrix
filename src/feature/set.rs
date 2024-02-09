// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use super::Feature;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::{
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
pub(crate) struct Set(BTreeSet<Feature>);

impl Display for Set {
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

impl FromIterator<Feature> for Set {
    fn from_iter<T: IntoIterator<Item = Feature>>(iter: T) -> Self {
        Set(iter.into_iter().collect())
    }
}

impl IntoIterator for Set {
    type Item = Feature;
    type IntoIter = <<Self as OpsDeref>::Target as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
