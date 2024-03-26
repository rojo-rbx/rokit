use std::{env::args, process::exit, str::FromStr};

use anyhow::{Context, Result};

use rokit::{
    storage::Home,
    system::{current_exe_name, run_interruptible},
    tool::ToolAlias,
};

use crate::util::discover_closest_tool_spec;

#[derive(Debug, Clone)]
pub struct Runner {
    exe_name: String,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            exe_name: current_exe_name(),
        }
    }

    pub fn should_run(&self) -> bool {
        self.exe_name != env!("CARGO_BIN_NAME")
    }

    pub async fn run(&self) -> Result<()> {
        let alias = ToolAlias::from_str(&self.exe_name)?;

        let home = Home::load_from_env().await?;
        let spec = discover_closest_tool_spec(&home, &alias)
            .await
            .with_context(|| format!("Failed to find tool '{alias}'"))?;

        let path = home.tool_storage().tool_path(&spec);
        let args = args().skip(1).collect::<Vec<_>>();

        let code = run_interruptible(&path, &args).await?;
        exit(code);
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}
