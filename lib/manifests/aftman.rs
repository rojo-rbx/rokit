use std::{path::Path, str::FromStr};

use toml_edit::DocumentMut;

use crate::{
    result::AftmanResult,
    util::{load_from_file_fallible, save_to_file},
};

const MANIFEST_FILE_NAME: &str = "aftman.toml";
const MANIFEST_DEFAULT_CONTENTS: &str = r#"
# This file lists tools managed by Aftman, a cross-platform toolchain manager.
# For more information, see <|REPOSITORY_URL|>

# New tools can be added by running `aftman add <tool>` in a terminal.
[tools]
"#;

/**
    Aftman manifest file.

    Lists tools managed by Aftman.
*/
#[derive(Debug, Clone)]
pub struct AftmanManifest {
    document: DocumentMut,
}

impl AftmanManifest {
    /**
        Loads the manifest from the given directory.

        This will search for a file named `aftman.toml` in the given directory.
    */
    pub async fn load(dir: impl AsRef<Path>) -> AftmanResult<Self> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        load_from_file_fallible(path).await
    }

    /**
        Saves the manifest to the given directory.

        This will write the manifest to a file named `aftman.toml` in the given directory.
    */
    pub async fn save(&self, dir: impl AsRef<Path>) -> AftmanResult<()> {
        let path = dir.as_ref().join(MANIFEST_FILE_NAME);
        save_to_file(path, self.clone()).await
    }

    // TODO: Add methods to interact with the manifest.
}

impl FromStr for AftmanManifest {
    type Err = toml_edit::TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let document = s.parse::<DocumentMut>()?;
        Ok(Self { document })
    }
}

impl ToString for AftmanManifest {
    fn to_string(&self) -> String {
        self.document.to_string()
    }
}

impl Default for AftmanManifest {
    fn default() -> Self {
        let document = MANIFEST_DEFAULT_CONTENTS
            .replace("<|REPOSITORY_URL|>", env!("CARGO_PKG_REPOSITORY"))
            .parse::<DocumentMut>()
            .unwrap();
        Self { document }
    }
}
