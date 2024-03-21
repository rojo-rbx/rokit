use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::env::current_dir;
use std::env::{consts::EXE_SUFFIX, current_exe};
use std::fmt::Write;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use futures::stream::FuturesUnordered;
use futures::StreamExt as _;
use itertools::Itertools;
use tokio::fs;
use tokio::task::spawn_blocking;
use tokio::time::Instant;

use crate::auth::AuthManifest;
use crate::home::Home;
use crate::manifest::Manifest;
use crate::tool_alias::ToolAlias;
use crate::tool_id::ToolId;
use crate::tool_name::ToolName;
use crate::tool_source::{Asset, GitHubSource, Release};
use crate::tool_spec::ToolSpec;
use crate::trust::{TrustCache, TrustMode, TrustStatus};

pub struct ToolStorage {
    pub storage_dir: PathBuf,
    pub bin_dir: PathBuf,
    home: Home,
    auth: Option<AuthManifest>,
    github: OnceCell<GitHubSource>,
}

impl ToolStorage {
    pub async fn new(home: &Home) -> anyhow::Result<Self> {
        let storage_dir = home.path().join("tool-storage");
        fs::create_dir_all(&storage_dir).await?;

        let bin_dir = home.path().join("bin");
        fs::create_dir_all(&bin_dir).await?;

        let auth = AuthManifest::load(home)?;

        Ok(Self {
            storage_dir,
            bin_dir,
            home: home.clone(),
            auth,
            github: OnceCell::new(),
        })
    }

    pub async fn add(
        &self,
        spec: &ToolSpec,
        alias: Option<&ToolAlias>,
        global: bool,
    ) -> anyhow::Result<()> {
        let current_dir = current_dir().context("Failed to find current working directory")?;

        let alias = match alias {
            Some(alias) => Cow::Borrowed(alias),
            None => Cow::Owned(ToolAlias::new(spec.name().name())?),
        };

        let id = self.install_inexact(spec, TrustMode::Check).await?;
        self.link(&alias).await?;

        if global {
            Manifest::add_global_tool(&self.home, &alias, &id)?;
        } else {
            Manifest::add_local_tool(&self.home, &current_dir, &alias, &id)?;
        }

        Ok(())
    }

    pub async fn run(&self, id: &ToolId, args: Vec<String>) -> anyhow::Result<i32> {
        self.install_exact(id, TrustMode::Check, false).await?;

        let exe_path = self.exe_path(id);
        let code = crate::process::run(&exe_path, args)
            .await
            .with_context(|| {
                format!("Failed to run tool {id}, your installation may be corrupt.")
            })?;
        Ok(code)
    }

    /// Update all executables managed by Aftman, which might include Aftman
    /// itself.
    pub async fn update_links(&self) -> anyhow::Result<()> {
        let self_path =
            current_exe().context("Failed to discover path to the Aftman executable")?;
        let self_name = self_path.file_name().unwrap();

        tracing::info!("Updating all Aftman binaries...");

        // Copy our current executable into a temp directory. That way, if it
        // ends up replaced by this process, we'll still have the file that
        // we're supposed to be copying.
        tracing::debug!("Copying own executable into temp dir");
        let source_dir = tempfile::tempdir()?;
        let source_path = source_dir.path().join(self_name);
        fs::copy(&self_path, &source_path).await?;
        let self_path = source_path;

        let junk_dir = tempfile::tempdir()?;
        let aftman_name = format!("aftman{EXE_SUFFIX}");
        let mut found_aftman = false;

        let mut read_dir = fs::read_dir(&self.bin_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();

            if name == aftman_name {
                found_aftman = true;
            }

            tracing::debug!("Updating {:?}", name);

            // Copy the executable into a temp directory so that we can replace
            // it even if it's currently running.
            fs::rename(&path, junk_dir.path().join(name)).await?;
            fs::copy(&self_path, path).await?;
        }

        // If we didn't find and update Aftman already, install it.
        if !found_aftman {
            tracing::info!("Installing Aftman...");
            let aftman_path = self.bin_dir.join(aftman_name);
            fs::copy(&self_path, aftman_path).await?;
        }

        tracing::info!("Updated Aftman binaries successfully!");

        Ok(())
    }

    /// Install all tools from all reachable manifest files.
    pub async fn install_all(
        &self,
        trust: TrustMode,
        force: bool,
        skip_untrusted: bool,
    ) -> anyhow::Result<()> {
        let current_dir = current_dir().context("Failed to get current working directory")?;
        let manifests = Manifest::discover(&self.home, &current_dir)?;

        // Installing all tools is split into multiple steps:
        // 1. Trust check, which may prompt the user and yield if untrusted
        // 2. Installation of trusted tools, which runs all installation concurrently
        // 3. Linking of installed tools to their alias files
        // 4. Reporting of installation trust errors, unless trust errors are skipped

        let mut trust_futs = FuturesUnordered::new();
        for (alias, tool_id) in manifests
            .into_iter()
            .flat_map(|manifest| manifest.tools.clone())
        {
            trust_futs.push(async move {
                self.trust_check(tool_id.name(), trust).await?;
                anyhow::Ok((alias, tool_id.clone()))
            });
        }

        let mut trusted_tools = Vec::new();
        let mut trust_errors = Vec::new();
        while let Some(result) = trust_futs.next().await {
            match result {
                Ok((alias, tool_id)) => trusted_tools.push((alias, tool_id)),
                Err(e) => trust_errors.push(e),
            }
        }

        let mut install_futs = FuturesUnordered::new();
        for (alias, tool_id) in &trusted_tools {
            install_futs.push(async {
                self.install_exact(tool_id, trust, force).await?;
                self.link(alias).await?;
                anyhow::Ok(())
            });
        }
        while let Some(result) = install_futs.next().await {
            result?;
        }

        if !trust_errors.is_empty() && !skip_untrusted {
            bail!(
                "Installation trust check failed for the following tools:\n{}",
                trust_errors.iter().map(|e| format!("    {e}")).join("\n")
            )
        }

        Ok(())
    }

    /// Ensure a tool that matches the given spec is installed.
    async fn install_inexact(&self, spec: &ToolSpec, trust: TrustMode) -> anyhow::Result<ToolId> {
        let installed_path = self.storage_dir.join("installed.txt");
        let installed = InstalledToolsCache::read(&installed_path).await?;

        self.trust_check(spec.name(), trust).await?;

        tracing::info!("Installing tool: {}", spec);

        tracing::debug!("Fetching GitHub releases...");
        let github = self
            .github
            .get_or_init(|| GitHubSource::new(self.auth.as_ref()));
        let mut releases = github.get_all_releases(spec.name()).await?;
        releases.sort_by(|a, b| a.version.cmp(&b.version).reverse());

        tracing::trace!("All releases found: {:#?}", releases);
        tracing::debug!("Choosing a release...");

        for release in &releases {
            // If we've requested a version, skip any releases that don't match
            // the request.
            if let Some(requested_version) = spec.version() {
                if requested_version != &release.version {
                    continue;
                }
            }

            let id = ToolId::new(spec.name().clone(), release.version.clone());

            if installed.tools.contains(&id) {
                tracing::debug!("Tool is already installed.");
                return Ok(id);
            }

            let mut compatible_assets = self.get_compatible_assets(release);
            if compatible_assets.is_empty() {
                tracing::warn!(
                    "Version {} was compatible, but had no assets compatible with your platform.",
                    release.version
                );
                continue;
            }

            self.sort_assets_by_preference(&mut compatible_assets);
            let asset = &compatible_assets[0];

            tracing::info!(
                "Downloading {} v{} ({})...",
                spec.name(),
                release.version,
                asset.name
            );
            let artifact = github.download_asset(&asset.url).await?;

            self.install_artifact(&id, artifact)
                .await
                .with_context(|| {
                    format!(
                        "Could not install asset {} from tool {} release v{}",
                        asset.name,
                        id.name(),
                        release.version
                    )
                })?;

            InstalledToolsCache::add(&installed_path, &id)
                .await
                .context("Could not write installed tools cache file")?;

            tracing::info!("{} v{} installed successfully.", id.name(), release.version);

            return Ok(id);
        }

        bail!("Could not find a compatible release for {spec}");
    }

    /// Ensure a tool with the given tool ID is installed.
    async fn install_exact(
        &self,
        id: &ToolId,
        trust: TrustMode,
        force: bool,
    ) -> anyhow::Result<()> {
        let installed_path = self.storage_dir.join("installed.txt");
        let installed = InstalledToolsCache::read(&installed_path).await?;
        let is_installed = installed.tools.contains(id);

        if is_installed && !force {
            return Ok(());
        }

        self.trust_check(id.name(), trust).await?;

        tracing::info!("Installing tool: {id}");

        tracing::debug!("Fetching GitHub release...");
        let github = self
            .github
            .get_or_init(|| GitHubSource::new(self.auth.as_ref()));
        let release = github.get_release(id).await?;

        let mut compatible_assets = self.get_compatible_assets(&release);
        if compatible_assets.is_empty() {
            bail!("Tool {id} was found, but no assets were compatible with your system.");
        }

        self.sort_assets_by_preference(&mut compatible_assets);
        let asset = &compatible_assets[0];

        tracing::info!(
            "Downloading {} v{} ({})...",
            id.name(),
            release.version,
            asset.name
        );
        let artifact = github.download_asset(&asset.url).await?;

        self.install_artifact(id, artifact).await.with_context(|| {
            format!(
                "Could not install asset {} from tool {} release v{}",
                asset.name,
                id.name(),
                release.version
            )
        })?;

        InstalledToolsCache::add(&installed_path, id)
            .await
            .context("Could not write installed tools cache file")?;

        tracing::info!("{} v{} installed successfully.", id.name(), release.version);

        Ok(())
    }

    /// Picks the best asset out of the list of assets.
    fn sort_assets_by_preference(&self, assets: &mut [Asset]) {
        assets.sort_by(|a, b| a.arch.cmp(&b.arch).then(a.toolchain.cmp(&b.toolchain)));
    }

    /// Returns a list of compatible assets from the given release.
    fn get_compatible_assets(&self, release: &Release) -> Vec<Asset> {
        // If any assets list an OS or architecture that's compatible with
        // ours, we want to make that part of our filter criteria.
        let any_has_os = release
            .assets
            .iter()
            .any(|asset| asset.os.map(|os| os.compatible()).unwrap_or(false));
        let any_has_arch = release
            .assets
            .iter()
            .any(|asset| asset.arch.is_some() && asset.compatible());

        release
            .assets
            .iter()
            .filter(|asset| {
                // If any release has an OS that matched, filter out any
                // releases that don't match.
                let compatible_os = asset.os.map(|os| os.compatible()).unwrap_or(false);
                if any_has_os && !compatible_os {
                    return false;
                }

                // If any release has an OS and an architecture that matched
                // our platform, filter out any releases that don't match.
                let compatible = asset.compatible();
                if any_has_os && any_has_arch && !compatible {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    async fn trust_check(&self, name: &ToolName, mode: TrustMode) -> anyhow::Result<()> {
        let status = self.trust_status(name).await?;

        if status == TrustStatus::NotTrusted {
            if mode == TrustMode::Check {
                // If the terminal isn't interactive, tell the user that they
                // need to open an interactive terminal to trust this tool.
                if !io::stderr().is_terminal() {
                    bail!(
                        "Tool {name} has never been installed. \
                         Run `aftman add {name}` in your terminal to install it and trust this tool.",
                    );
                }

                // Since the terminal is interactive, ask the user if they're
                // sure they want to install this tool.
                let proceed = dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "Tool {} has never been installed before. Install it?",
                        name
                    ))
                    .interact_opt()?;

                if let Some(false) | None = proceed {
                    bail!(
                        "Tool {name} is not trusted. \
                         Run `aftman trust {name}` in your terminal to trust it.",
                    );
                }
            }

            TrustCache::add(&self.home, name.clone()).await?;
        }

        Ok(())
    }

    async fn trust_status(&self, name: &ToolName) -> anyhow::Result<TrustStatus> {
        let trusted = TrustCache::read(&self.home).await?;
        let is_trusted = trusted.tools.contains(name);
        if is_trusted {
            Ok(TrustStatus::Trusted)
        } else {
            Ok(TrustStatus::NotTrusted)
        }
    }

    async fn install_executable(&self, id: &ToolId, contents: &[u8]) -> anyhow::Result<()> {
        let output_path = self.exe_path(id);

        fs::create_dir_all(output_path.parent().unwrap()).await?;
        fs::write(&output_path, contents).await?;

        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            use tokio::fs::set_permissions;

            set_permissions(&output_path, Permissions::from_mode(0o755))
                .await
                .context("failed to mark executable as executable")?;
        }

        Ok(())
    }

    async fn install_artifact(&self, id: &ToolId, artifact: Vec<u8>) -> anyhow::Result<()> {
        let output_path = self.exe_path(id);
        let expected_name = format!("{}{EXE_SUFFIX}", id.name().name());

        fs::create_dir_all(output_path.parent().unwrap()).await?;

        // Reading a zip file can be slow, so we spawn a blocking task for it.
        let zipped_id = id.clone();
        let zipped_file = spawn_blocking(move || {
            let start = Instant::now();

            let mut found = None;
            let mut reader = io::Cursor::new(&artifact);
            let mut zip = zip::ZipArchive::new(&mut reader)?;

            // If there is an executable with an exact name match, install that one.
            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;

                if file.name() == expected_name {
                    let mut bytes = Vec::new();
                    let path = file.name().to_string();
                    file.read_to_end(&mut bytes)?;
                    found = Some((path, bytes));
                }
            }

            if found.is_none() {
                // ...otherwise, look for any file with the system's EXE_SUFFIX and
                // install that.
                for i in 0..zip.len() {
                    let mut file = zip.by_index(i)?;

                    if file.name().ends_with(EXE_SUFFIX) {
                        let mut bytes = Vec::new();
                        let path = file.name().to_string();
                        file.read_to_end(&mut bytes)?;
                        found = Some((path, bytes));
                    }
                }
            }

            tracing::debug!("Read zip file for {zipped_id} in {:?}", start.elapsed());
            anyhow::Ok(found)
        })
        .await??;

        if let Some((path, bytes)) = zipped_file {
            tracing::debug!("Installing file {path} from archive...");
            self.install_executable(id, &bytes).await?;
            Ok(())
        } else {
            bail!("no executables were found in archive");
        }
    }

    async fn link(&self, alias: &ToolAlias) -> anyhow::Result<()> {
        let self_path =
            current_exe().context("Failed to discover path to the Aftman executable")?;

        let link_name = format!("{}{}", alias.as_ref(), EXE_SUFFIX);
        let link_path = self.bin_dir.join(link_name);

        fs::copy(self_path, link_path)
            .await
            .context("Failed to create Aftman alias")?;
        Ok(())
    }

    fn exe_path(&self, id: &ToolId) -> PathBuf {
        let mut dir = self.storage_dir.clone();
        dir.push(id.name().scope());
        dir.push(id.name().name());
        dir.push(id.version().to_string());
        dir.push(format!("{}{}", id.name().name(), EXE_SUFFIX));
        dir
    }
}

#[derive(Debug)]
pub struct InstalledToolsCache {
    pub tools: BTreeSet<ToolId>,
}

impl InstalledToolsCache {
    pub async fn read(path: &Path) -> anyhow::Result<Self> {
        let contents = match fs::read_to_string(path).await {
            Ok(v) => v,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    String::new()
                } else {
                    bail!(err);
                }
            }
        };

        let tools = contents
            .lines()
            .filter_map(|line| line.parse::<ToolId>().ok())
            .collect();

        Ok(Self { tools })
    }

    pub async fn add(path: &Path, id: &ToolId) -> anyhow::Result<()> {
        let mut cache = Self::read(path).await?;
        cache.tools.insert(id.clone());

        let mut output = String::new();
        for tool in cache.tools {
            writeln!(&mut output, "{}", tool).unwrap();
        }

        fs::write(path, output).await?;
        Ok(())
    }
}
