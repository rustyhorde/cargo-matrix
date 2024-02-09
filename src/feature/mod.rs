// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

mod matrix;
mod set;

use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

pub(crate) use self::matrix::Matrix as FeatureMatrix;
pub(crate) use self::set::Set as FeatureSet;

#[derive(
    Clone,
    Debug,
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
#[as_ref(forward)]
#[as_mut(forward)]
pub(crate) struct Feature<'a>(pub(crate) Cow<'a, str>);

impl Display for Feature<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<String> for Feature<'static> {
    fn from(s: String) -> Self {
        Feature(Cow::Owned(s))
    }
}

impl<'a> From<&'a str> for Feature<'a> {
    fn from(s: &'a str) -> Self {
        Feature(Cow::Borrowed(s))
    }
}
