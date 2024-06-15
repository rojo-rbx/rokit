#![allow(clippy::to_string_trait_impl)]
// NOTE: We don't want to implement Display here since it may
// make library consumers think that auth manifests are meant
// to be displayed - they are only meant to be stringified.

use std::{collections::HashMap, path::Path, str::FromStr};

use toml_edit::{DocumentMut, Formatted, Item, Value};
use tracing::warn;

use crate::{
    result::{RokitError, RokitResult},
    sources::ArtifactProvider,
    util::fs::{load_from_file, save_to_file},
};

pub const MANIFEST_FILE_NAME: &str = "auth.toml";
pub(super) const MANIFEST_DEFAULT_CONTENTS: &str = "
# This file lists authentication tokens managed by Rokit, a toolchain manager for Roblox projects.
# For more information, see <|REPOSITORY_URL|>

# github = \"ghp_tokenabcdef1234567890\"
";

/**
    Authentication manifest file.

    Contains authentication tokens managed by Rokit.
*/
#[derive(Debug, Clone)]
pub struct AuthManifest {
    document: DocumentMut,
}

impl AuthManifest {
    /**
        Loads the manifest from the given directory, or creates a new one if it doesn't exist.

        If the manifest doesn't exist, a new one will be created with default contents and saved.

        See [`AuthManifest::load`] and [`AuthManifest::save`] for more information.

        # Errors

        - If the manifest file could not be loaded or created.
    */
    pub async fn load_or_create(dir: impl AsRef<Path>) -> RokitResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        match load_from_file(path).await {
            Ok(manifest) => Ok(manifest),
            Err(RokitError::FileNotFound(_)) => {
                let new = Self::default();
                new.save(dir).await?;
                Ok(new)
            }
            Err(e) => Err(e),
        }
    }

    /**
        Loads the manifest from the given directory.

        This will search for a file named `auth.toml` in the given directory.

        # Errors

        - If the manifest file could not be loaded.
    */
    #[tracing::instrument(skip(dir), level = "trace")]
    pub async fn load(dir: impl AsRef<Path>) -> RokitResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Loading manifest");
        load_from_file(path).await
    }

    /**
        Saves the manifest to the given directory.

        This will write the manifest to a file named `auth.toml` in the given directory.

        # Errors

        - If the manifest file could not be saved.
    */
    #[tracing::instrument(skip(self, dir), level = "trace")]
    pub async fn save(&self, dir: impl AsRef<Path>) -> RokitResult<()> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        tracing::trace!(?path, "Saving manifest");
        save_to_file(path, self.clone()).await
    }

    /**
        Checks if the manifest contains an authentication token for the given artifact provider.
    */
    #[must_use]
    pub fn has_token(&self, artifact_provider: ArtifactProvider) -> bool {
        self.document.contains_key(artifact_provider.as_str())
    }

    /**
        Gets the authentication token for the given artifact provider.

        Returns `None` if the token is not present.
    */
    #[must_use]
    pub fn get_token(&self, artifact_provider: ArtifactProvider) -> Option<String> {
        let token = self.document.get(artifact_provider.as_str())?;
        token.as_str().map(ToString::to_string)
    }

    /**
        Gets all authentication tokens found in the manifest.
    */
    #[must_use]
    pub fn get_all_tokens(&self) -> HashMap<ArtifactProvider, String> {
        self.document
            .iter()
            .filter_map(|(key, value)| {
                let provider = ArtifactProvider::from_str(key).ok()?;
                let token = value.as_str()?.to_string();
                Some((provider, token))
            })
            .collect()
    }

    /**
        Sets the authentication token for the given artifact provider.

        Returns `true` if the token replaced an older
        one, `false` if an older token was not present.
    */
    #[must_use]
    pub fn set_token(
        &mut self,
        artifact_provider: ArtifactProvider,
        token: impl Into<String>,
    ) -> bool {
        let tab = self.document.as_table_mut();
        let old = tab.insert(
            artifact_provider.as_str(),
            Item::Value(Value::String(Formatted::new(token.into()))),
        );
        old.is_some()
    }

    /**
        Unsets the authentication token for the given artifact provider.

        Returns `true` if the token was removed, `false` if it was not present.
    */
    #[must_use]
    pub fn unset_token(&mut self, artifact_provider: ArtifactProvider) -> bool {
        let tab = self.document.as_table_mut();
        tab.remove(artifact_provider.as_str()).is_some()
    }
}

impl FromStr for AuthManifest {
    type Err = toml_edit::TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let document = s.parse::<DocumentMut>()?;

        /*
            Check for invalid auth entries and warn the user about them
            as a preprocessing step. We do this here instead of when accessed
            in manifest methods to avoid duplicate warnings being emitted.
        */
        for (key, value) in document.iter() {
            if let Err(e) = ArtifactProvider::from_str(key) {
                warn!(
                    "Encountered unknown artifact provider '{}' in auth manifest!\
                    \nError: {e}",
                    key
                );
            }
            if !value.is_str() {
                warn!(
                    "Encountered invalid value for artifact provider '{}' in auth manifest!\
                    \nExpected: String\
                    \nActual: {}",
                    key,
                    value.type_name()
                );
            }
        }

        Ok(Self { document })
    }
}

impl ToString for AuthManifest {
    fn to_string(&self) -> String {
        self.document.to_string()
    }
}

impl Default for AuthManifest {
    fn default() -> Self {
        let document = super::make_manifest_template(MANIFEST_DEFAULT_CONTENTS)
            .parse::<DocumentMut>()
            .expect("default manifest template should be valid");
        Self { document }
    }
}
