use anyhow::{bail, Context, Result};
use clap::Parser;

use console::style;
use rokit::{
    manifests::AuthManifest,
    sources::{github::GithubProvider, ArtifactProvider},
    storage::Home,
};

use crate::util::CliProgressTracker;

/// Authenticate with an artifact provider, such as GitHub.
#[derive(Debug, Parser)]
pub struct AuthenticateSubcommand {
    /// The artifact / tool provider to authenticate with.
    pub provider: ArtifactProvider,
    /// The token to use for authentication.
    /// Can be omitted when removing a token.
    #[clap(long)]
    pub token: Option<String>,
    /// If the token should be removed.
    #[clap(long, default_value = "false")]
    pub remove: bool,
    /// If parsing validation should be skipped when adding a new token.
    #[clap(long, default_value = "false")]
    pub skip_parse: bool,
    /// If live API verification should be skipped when adding a new token.
    #[clap(long, default_value = "false")]
    pub skip_verify: bool,
}

impl AuthenticateSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let pt = CliProgressTracker::new_with_message(
            "Authenticating",
            if self.token.is_some() { 4 } else { 3 },
        );

        let mut auth = AuthManifest::load_or_create(home.path())
            .await
            .context("Failed to load or create auth manifest")?;
        pt.task_completed();

        let styled_provider = style(self.provider.display_name())
            .bold()
            .white()
            .to_string();
        let styled_add_command = style(format!(
            "rokit authenticate {} --token YOUR_TOKEN_HERE",
            self.provider
        ))
        .bold()
        .green()
        .to_string();
        let styled_remove_command = style(format!("rokit authenticate {} --remove", self.provider))
            .bold()
            .green()
            .to_string();

        let exists = auth.has_token(self.provider);
        if self.remove {
            if !exists {
                bail!(
                    "No authentication token for {styled_provider} exists.\
                    \nRun `{styled_add_command}` to add a new token."
                );
            }
        } else if exists {
            bail!(
                "An authentication token for {styled_provider} already exists.\
                \nRun `{styled_remove_command}` to remove it and allow adding a new token.",
            );
        }

        if self.remove {
            let was_removed = auth.unset_token(self.provider);
            assert!(was_removed, "token was not removed");
        } else if let Some(token) = self.token {
            let token = token.trim().to_string();

            pt.update_message("Verifying");
            verify_token(self.provider, &token, self.skip_parse, self.skip_verify).await?;
            pt.task_completed();

            let had_token = auth.set_token(self.provider, token);
            assert!(!had_token, "token was overwritten");
        } else {
            bail!(
                "A token must be given to authenticate with {styled_provider}.\
                \nExample usage: `{styled_add_command}`"
            )
        }

        pt.update_message("Saving");
        auth.save(home.path()).await?;

        pt.finish_with_emoji_and_message(
            "✓",
            format!(
                "{}{} {styled_provider} authentication successfully. {}",
                if self.remove { "Removed" } else { "Added" },
                if !self.remove && !self.skip_verify {
                    " and verified"
                } else {
                    ""
                },
                pt.formatted_elapsed()
            ),
        );

        Ok(())
    }
}

async fn verify_token(
    provider: ArtifactProvider,
    token: &str,
    skip_parse: bool,
    skip_verify: bool,
) -> Result<()> {
    // Verify the formatting of the token, if desired.
    if !skip_parse {
        let validated = match provider {
            ArtifactProvider::GitHub => {
                is_gh_classic_token(token) || is_gh_fine_grained_token(token)
            }
        };

        if !validated {
            let bullet = style("•").dim();
            let valid_formats = match provider {
                ArtifactProvider::GitHub => vec![
                    format!("{bullet} Starting with 'gh' followed by a lowercase letter and an underscore"),
                    format!("{bullet} Starting with 'github_pat_'"),
                ],
            };

            let styled_flag = style("--skip-parse").bold().green();
            let styled_command = style(format!(
                "rokit authenticate {provider} --token YOUR_TOKEN_HERE --skip-parse"
            ))
            .bold()
            .green();

            bail!(
                "Failed to verify the provided {0} token format.\
                \nPlease ensure the validity of the token, or generate a new token.\
                \n\
                \nValid formats for {0} are:\
                \n{1}\
                \n\
                \nIf you are certain the token is valid, you may skip parsing.\
                \nTo skip parsing, run the command again with the `{styled_flag}` flag.\
                \nExample usage: {styled_command}",
                provider.display_name(),
                valid_formats.join("\n"),
            )
        }
    }

    // Verify the actual validity of the token, if desired.
    if !skip_verify {
        let verified = match provider {
            ArtifactProvider::GitHub => {
                let client = GithubProvider::new_authenticated(token)?;
                let verify_res = client.verify_authentication().await;
                verify_res.context("GitHub API returned an error during token verification")?
            }
        };

        if !verified {
            let styled_flag = style("--skip-verify").bold().green();
            let styled_command = style(format!(
                "rokit authenticate {provider} --token YOUR_TOKEN_HERE --skip-verify"
            ))
            .bold()
            .green();
            bail!(
                "Failed to verify the provided {0} token using the API.\
                \nPlease ensure the validity of the token, or generate a new token.\
                \n\
                \nIf you are certain the token is valid, you may skip verification.\
                \nTo skip verification, run the command again with the `{styled_flag}` flag.\
                \nExample usage: {styled_command}",
                provider.display_name(),
            )
        }
    }

    Ok(())
}

fn is_gh_classic_token(token: &str) -> bool {
    match token.chars().take(4).collect::<Vec<_>>().as_slice() {
        ['g', 'h', c, '_'] => c.is_ascii_alphabetic() && c.is_ascii_lowercase(),
        ['g', 'h', '_', _] => true,
        _ => false,
    }
}

fn is_gh_fine_grained_token(token: &str) -> bool {
    token.starts_with("github_pat_")
}
