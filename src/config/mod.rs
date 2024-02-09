// Copyright (c) 2024 cargo-matrix developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use crate::feature::{FeatureMatrix, FeatureSet};
use anyhow::{anyhow, Result};
use figment::{
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Deserialize, Getters, MutGetters, Serialize, Setters)]
#[getset(get = "pub(crate)")]
#[serde(bound(deserialize = "'de: 'c"))]
pub(crate) struct Config<'c> {
    #[getset(get_mut = "pub(crate)")]
    channel: Vec<Channel<'c>>,
}

impl Config<'_> {
    pub(crate) fn from<T: Provider>(provider: T) -> Result<Self> {
        Ok(Figment::from(provider).extract()?)
    }

    pub(crate) fn seed(&self, channel: &str) -> Result<Option<FeatureSet<'_>>> {
        if let Some(seed) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .seed()
        {
            Ok(Some(seed.clone()))
        } else {
            Ok(self.get_default()?.seed().clone())
        }
    }

    pub(crate) fn include(&self, channel: &str) -> Result<FeatureSet<'_>> {
        if let Some(include) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .include()
        {
            Ok(include.clone())
        } else {
            Ok(self.get_default()?.include().clone().unwrap_or_default())
        }
    }

    pub(crate) fn deny(&self, channel: &str) -> Result<FeatureSet<'_>> {
        if let Some(deny) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .deny()
        {
            Ok(deny.clone())
        } else {
            Ok(self.get_default()?.deny().clone().unwrap_or_default())
        }
    }

    pub(crate) fn skip(&self, channel: &str) -> Result<FeatureMatrix<'_>> {
        if let Some(skip) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .skip()
        {
            Ok(skip.clone())
        } else {
            Ok(self.get_default()?.skip().clone().unwrap_or_default())
        }
    }

    pub(crate) fn include_hidden(&self, channel: &str) -> Result<bool> {
        if let Some(include_hidden) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .include_hidden()
        {
            Ok(include_hidden.clone())
        } else {
            Ok(self
                .get_default()?
                .include_hidden()
                .clone()
                .unwrap_or_default())
        }
    }

    pub(crate) fn include_optional(&self, channel: &str) -> Result<bool> {
        if let Some(include_optional) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .include_optional()
        {
            Ok(include_optional.clone())
        } else {
            Ok(self
                .get_default()?
                .include_optional()
                .clone()
                .unwrap_or_default())
        }
    }

    pub(crate) fn get_default(&self) -> Result<&'_ Channel<'_>> {
        self.get_channel("default")
    }

    fn get_channel(&self, channel: &str) -> Result<&'_ Channel<'_>> {
        self.channel
            .iter()
            .find(|c| c.name() == channel)
            .ok_or_else(|| anyhow!(format!("channel '{channel}' not defined")))
    }
}

impl Default for Config<'_> {
    fn default() -> Self {
        let mut default_channel = Channel::default();
        default_channel.name = Cow::Borrowed("default");
        Self {
            channel: vec![default_channel],
        }
    }
}

impl Provider for Config<'_> {
    fn metadata(&self) -> Metadata {
        Metadata::named("config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(self).data()
    }
}

#[derive(Debug, Default, Deserialize, Getters, Serialize, Setters)]
#[getset(get = "pub(crate)")]
#[serde(bound(deserialize = "'de: 'c"))]
pub(crate) struct Channel<'c> {
    name: Cow<'c, str>,

    /// If this set is not empty, only these features will be used to construct the
    /// matrix.
    seed: Option<FeatureSet<'c>>,

    /// All of these features will be included in every feature set in the matrix.
    include: Option<FeatureSet<'c>>,

    /// Any feature set that includes any of these will be excluded from the matrix.
    /// This includes features enabled by other features.
    ///
    /// This can be used for things like having an "__unstable" feature that gets
    /// enabled by any other features that use unstable rust features and then
    /// excluding "__unstable" if not on nightly.
    #[getset(set = "pub(crate)")]
    deny: Option<FeatureSet<'c>>,

    /// These sets will be dropped from the matrix.
    skip: Option<FeatureMatrix<'c>>,

    /// Some crates prepend internal features with a double underscore. If this
    /// flag is not set, those features will not be used to build the matrix, but
    /// will be allowed if they are enabled by other features.
    include_hidden: Option<bool>,

    include_optional: Option<bool>,
}
