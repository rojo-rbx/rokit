use std::{
    env::{args, consts::EXE_EXTENSION},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

use anyhow::{Context, Result};

use aftman::{storage::Home, system::run_interruptible, tool::ToolAlias};

use crate::util::discover_closest_tool_spec;

#[derive(Debug, Clone)]
pub struct Runner;

impl Runner {
    pub fn new() -> Self {
        Self
    }

    pub fn arg0_file_name(&self) -> String {
        let arg0 = args().next().unwrap();
        let exe_path = PathBuf::from(arg0);
        let exe_name = exe_path
            .file_name()
            .expect("Invalid file name passed as arg0")
            .to_str()
            .expect("Non-UTF8 file name passed as arg0")
            .trim_end_matches(EXE_EXTENSION);
        exe_name.to_string()
    }

    pub async fn run(&self, alias: impl AsRef<str>) -> Result<()> {
        let alias = ToolAlias::from_str(alias.as_ref())?;

        let home = Home::load_from_env().await?;

        let result = async {
            let spec = discover_closest_tool_spec(&home, &alias)
                .await
                .with_context(|| format!("Failed to find tool '{alias}'"))?;
            let path = home.tool_storage().tool_path(&spec);
            let args = args().skip(1).collect::<Vec<_>>();
            anyhow::Ok(run_interruptible(&path, &args).await?)
        }
        .await;

        home.save().await?;

        exit(result?);
    }
}
