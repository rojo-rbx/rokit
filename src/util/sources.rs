use anyhow::Result;

use rokit::{
    manifests::AuthManifest,
    sources::{ArtifactProvider, GitHubSource},
    storage::Home,
};

pub async fn github_tool_source(home: &Home) -> Result<GitHubSource> {
    // We might be wanting to add a private tool, so load our tool source with auth
    // FUTURE: Some kind of generic solution for tool sources and auth for them
    let auth = AuthManifest::load_or_create(home.path()).await?;
    Ok(match auth.get_token(ArtifactProvider::GitHub) {
        Some(token) => GitHubSource::new_authenticated(token)?,
        None => GitHubSource::new()?,
    })
}
