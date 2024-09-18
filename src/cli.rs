use anyhow::Result;
use clap::Parser;

use crate::{_use, config::FgmConfig, env, install, list_installed, list_remote, uninstall};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub sub: Subcommand,
}

#[derive(Parser, Debug, Clone)]
pub enum Subcommand {
    /// Install a specific version of Go
    Install { version: String },
    /// List installed versions
    List {
        // sort
        #[clap(short, long)]
        sort: bool,
    },
    /// List all remote versions
    #[clap(name = "list-remote")]
    LsRemote {
        // sort
        #[clap(short, long)]
        sort: bool,
    },
    /// Uninstall a specific version of Go
    Uninstall { version: String },
    /// Use a specific version of Go
    Use { version: String },
    /// Print and set up required environment variables for fgm
    ///
    /// This command generates a series of shell commands that
    /// should be evaluated by your shell to create a fgm-ready environment.
    ///
    /// Each shell has its own syntax of evaluating a dynamic expression.
    /// For example, evaluating fgm on Bash and Zsh would look like `eval "$(fgm env)"`.
    ///
    /// Now, only Bash and Zsh are supported.It may also work on other shells that support the `export` command.
    Env,
}

impl Subcommand {
    pub fn run(&self, config: &FgmConfig) -> Result<()> {
        match self {
            Subcommand::Install { version } => {
                install(config, version)?;
            }
            Subcommand::List { sort } => {
                list_installed(config, *sort);
            }
            Subcommand::LsRemote { sort } => {
                list_remote(config, *sort)?;
            }
            Subcommand::Uninstall { version } => {
                uninstall(config, version)?;
            }
            Subcommand::Use { version } => {
                _use(config, version)?;
            }
            Subcommand::Env => println!("{}", env(config)),
        }
        Ok(())
    }
}
