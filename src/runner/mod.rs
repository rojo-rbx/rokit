use std::{
    env::{args, consts::EXE_EXTENSION},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

use anyhow::{Context, Result};

use rokit::{storage::Home, system::run_interruptible, tool::ToolAlias};

use crate::util::discover_closest_tool_spec;

#[derive(Debug, Clone)]
pub struct Runner {
    exe_name: String,
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_run(&self) -> bool {
        self.exe_name != env!("CARGO_PKG_NAME")
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
        let arg0 = args().next().unwrap();
        let exe_path = PathBuf::from(arg0);
        let exe_name = exe_path
            .file_name()
            .expect("Invalid file name passed as arg0")
            .to_str()
            .expect("Non-UTF8 file name passed as arg0")
            .trim_end_matches(EXE_EXTENSION);
        Self {
            exe_name: exe_name.to_string(),
        }
    }
}
