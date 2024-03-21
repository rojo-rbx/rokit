use anyhow::Result;
use clap::Parser;

mod list;

use self::list::ListSubcommand;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        self.subcommand.run().await
    }
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    List(ListSubcommand),
}

impl Subcommand {
    pub async fn run(self) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.run().await,
        }
    }
}
