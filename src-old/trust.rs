use std::collections::BTreeSet;
use std::fmt::Write;
use std::io;

use anyhow::bail;
use tokio::fs;

use crate::home::Home;
use crate::tool_name::ToolName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustMode {
    Check,
    NoCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    Trusted,
    NotTrusted,
}

#[derive(Debug)]
pub struct TrustCache {
    pub tools: BTreeSet<ToolName>,
}

impl TrustCache {
    pub async fn read(home: &Home) -> anyhow::Result<Self> {
        let path = home.path().join("trusted.txt");

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
            .filter_map(|line| line.parse::<ToolName>().ok())
            .collect();

        Ok(Self { tools })
    }

    pub async fn add(home: &Home, name: ToolName) -> anyhow::Result<bool> {
        let mut cache = Self::read(home).await?;

        if cache.tools.insert(name) {
            let mut output = String::new();
            for tool in cache.tools {
                writeln!(&mut output, "{}", tool).unwrap();
            }

            let path = home.path().join("trusted.txt");
            fs::write(path, output).await?;

            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_and_add() -> anyhow::Result<()> {
        let home = Home::new_temp()?;

        let cache = TrustCache::read(&home).await?;
        assert!(cache.tools.is_empty());

        let tool_name: ToolName = "foo/bar".parse()?;

        let added = TrustCache::add(&home, tool_name.clone()).await?;
        assert!(added);

        let cache = TrustCache::read(&home).await?;
        assert!(cache.tools.len() == 1);
        assert!(cache.tools.contains(&tool_name));

        Ok(())
    }
}
