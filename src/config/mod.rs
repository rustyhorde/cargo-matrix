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

#[derive(Clone, Debug, Deserialize, Getters, MutGetters, Serialize, Setters)]
#[getset(get = "pub(crate)")]
pub(crate) struct Config {
    #[getset(get_mut = "pub(crate)")]
    channel: Vec<Channel>,
}

impl Config {
    pub(crate) fn from<T: Provider>(provider: T) -> Result<Self> {
        Ok(Figment::from(provider).extract()?)
    }

    pub(crate) fn seed(&self, channel: &str) -> Result<Option<FeatureSet>> {
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

    pub(crate) fn always_include(&self, channel: &str) -> Result<FeatureSet> {
        if let Some(always_include) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .always_include()
        {
            Ok(always_include.clone())
        } else {
            Ok(self
                .get_default()?
                .always_include()
                .clone()
                .unwrap_or_default())
        }
    }

    pub(crate) fn always_deny(&self, channel: &str) -> Result<FeatureSet> {
        if let Some(always_deny) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .always_deny()
        {
            Ok(always_deny.clone())
        } else {
            Ok(self
                .get_default()?
                .always_deny()
                .clone()
                .unwrap_or_default())
        }
    }

    pub(crate) fn skip(&self, channel: &str) -> Result<FeatureMatrix> {
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
            Ok(*include_hidden)
        } else {
            Ok(self.get_default()?.include_hidden().unwrap_or_default())
        }
    }

    pub(crate) fn include_all_optional(&self, channel: &str) -> Result<bool> {
        if let Some(include_all_optional) = self
            .get_channel(channel)
            .or_else(|_| self.get_default())?
            .include_all_optional()
        {
            Ok(*include_all_optional)
        } else {
            Ok(self
                .get_default()?
                .include_all_optional()
                .unwrap_or_default())
        }
    }

    pub(crate) fn include_optional(&self, channel: &str) -> Result<FeatureSet> {
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

    fn get_default(&self) -> Result<&'_ Channel> {
        self.get_channel("default")
    }

    fn get_channel(&self, channel: &str) -> Result<&'_ Channel> {
        self.channel
            .iter()
            .find(|c| c.name() == channel)
            .ok_or_else(|| anyhow!(format!("channel '{channel}' not defined")))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channel: vec![Channel {
                name: "default".to_string(),
                ..Default::default()
            }],
        }
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(self).data()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Getters, Serialize)]
#[getset(get = "pub(crate)")]
pub(crate) struct Channel {
    name: String,

    /// If this set is not empty, only these features will be used to construct the
    /// matrix.
    seed: Option<FeatureSet>,

    /// All of these features will be included in every feature set in the matrix.
    always_include: Option<FeatureSet>,

    /// Any feature set that includes any of these will be excluded from the matrix.
    /// This includes features enabled by other features.
    always_deny: Option<FeatureSet>,

    /// These sets will be dropped from the matrix.
    skip: Option<FeatureMatrix>,

    /// Some crates prepend internal features with a double underscore. If this
    /// flag is not set, those features will not be used to build the matrix, but
    /// will be allowed if they are enabled by other features.
    include_hidden: Option<bool>,

    /// Include all optional dependencies
    include_all_optional: Option<bool>,

    /// Include specific optional dependencies.
    /// This is independent of the `include_all_optional` setting.
    include_optional: Option<FeatureSet>,
}
