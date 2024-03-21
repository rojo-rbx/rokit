use std::io;

use anyhow::{bail, format_err, Context};
use serde::{Deserialize, Serialize};
use toml_edit::Document;

use crate::config::write_if_not_exists;
use crate::home::Home;

pub static MANIFEST_FILE_NAME: &str = "auth.toml";

static DEFAULT_MANIFEST: &str = r#"
# This file is for auth tokens managed by Aftman, a cross-platform toolchain manager.
# For more information, see https://github.com/LPGhatguy/aftman

# github = "token"
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthManifest {
    pub github: Option<String>,
}

impl AuthManifest {
    /// Create an empty global auth manifest if there isn't one already.
    pub fn init(home: &Home) -> anyhow::Result<()> {
        let base_dir = home.path();
        fs_err::create_dir_all(base_dir)?;

        let manifest_path = base_dir.join(MANIFEST_FILE_NAME);
        write_if_not_exists(&manifest_path, DEFAULT_MANIFEST.trim())?;

        Ok(())
    }

    /// Try to load an auth.toml from a directory
    pub fn load(home: &Home) -> anyhow::Result<Option<AuthManifest>> {
        let file_path = home.path().join(MANIFEST_FILE_NAME);

        let contents = match fs_err::read(&file_path) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }

                bail!(err);
            }
        };

        let manifest: AuthManifest = toml::from_slice(&contents)
            .with_context(|| format_err!("Invalid auth.toml at {}", file_path.display()))?;

        Ok(Some(manifest))
    }

    fn _add_token(home: &Home, token_type: &str, token: &str) -> anyhow::Result<()> {
        let manifest_path = home.path().join(MANIFEST_FILE_NAME);
        let content = fs_err::read_to_string(&manifest_path)?;
        let mut document: Document = content.parse()?;
        document[token_type] = toml_edit::value(token.to_string());

        fs_err::write(&manifest_path, document.to_string())?;

        log::info!("A {token_type} token has been added globally.");

        Ok(())
    }
}
