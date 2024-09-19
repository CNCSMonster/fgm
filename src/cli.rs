use anyhow::Result;
use clap::Parser;

use crate::{
    _use,
    config::{count_config_path, count_remotes_index_path, FgmContext},
    current_version, init_script, install, list_installed, list_remote, uninstall, update,
};

#[derive(Parser, Debug)]
#[clap(name = "fgm", version = env!("CARGO_PKG_VERSION"), bin_name = "fgm",about,long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub sub: Subcommand,
}

#[derive(Parser, Debug, Clone)]
pub enum Subcommand {
    /// Install a specific version of Go
    Install {
        version: String,
    },
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
    Uninstall {
        version: String,
    },
    /// Use a specific version of Go
    Use {
        version: String,
    },
    /// Print and set up required environment variables for fgm
    ///
    /// This command generates a series of shell commands that
    /// should be evaluated by your shell to create a fgm-ready environment.
    ///
    /// Each shell has its own syntax of evaluating a dynamic expression.
    /// For example, evaluating fgm on Bash and Zsh would look like `eval "$(fgm init)"`.
    ///
    /// Now, only Bash and Zsh are supported.It may also work on other shells that support the `export` command.
    Init,
    Current,
    /// show the runtime configuration
    Config,
    /// Update the fgm remotes index
    Update,
}

impl Subcommand {
    pub fn run(&self, ctx: &FgmContext) -> Result<()> {
        match self {
            Subcommand::Install { version } => {
                install(ctx, version)?;
            }
            Subcommand::List { sort } => {
                list_installed(ctx, *sort);
            }
            Subcommand::LsRemote { sort } => {
                list_remote(ctx, *sort)?;
            }
            Subcommand::Uninstall { version } => {
                uninstall(ctx, version)?;
            }
            Subcommand::Use { version } => {
                _use(ctx, version)?;
            }
            Subcommand::Init => println!("{}", init_script(ctx)),
            Subcommand::Current => println!(
                "{}",
                current_version(ctx).unwrap_or("not version selected".to_owned())
            ),
            Subcommand::Config => {
                println!("## Config Paths\n");
                println!("config_path: {:?}", count_config_path().ok());
                println!("remotes_index: {:?}", count_remotes_index_path().ok());

                println!("\n## Configurations\n");
                println!("installations_dir: {}", ctx.installations_dir);
                println!("gate_path: {}", ctx.gate_path);
                println!("remote_source: {}", ctx.remote_source);
            }
            Subcommand::Update => {
                update(ctx)?;
            }
        }
        Ok(())
    }
}
