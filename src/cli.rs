use anyhow::Result;
use clap::Parser;

use crate::{
    _use,
    config::{count_config_path, count_remotes_index_path, FgmContext},
    current_version, gen_completions, init_script, install, list_installed, list_remote, uninstall,
    update,
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
        #[clap(short, long)]
        update: bool,
    },
    /// List installed versions
    #[clap(name = "list",visible_aliases = &["ls"])]
    LsLocal {
        // sort
        #[clap(short, long)]
        sort: bool,
        // reverse
        #[clap(short, long)]
        reverse: bool,
    },
    /// List all remote versions
    #[clap(name = "list-remote",visible_aliases = &["ls-remote"])]
    LsRemote {
        // sort
        #[clap(short, long)]
        sort: bool,
        // reverse
        #[clap(short, long)]
        reverse: bool,
        // update cache
        #[clap(short, long)]
        update: bool,
    },
    /// Uninstall a specific version of Go
    Uninstall {
        version: String,
    },
    /// Use a specific version of Go
    Use {
        version: String,
        #[clap(short, long)]
        update: bool,
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
    /// Generate shell completions, which can be evaluated to enable shell completions for fgm
    ///
    /// The `shell` argument specifies the shell for which completions should be generated.
    ///
    /// Supported shells are:
    /// zsh, bash, fish, powershell, and elvish
    ///
    /// For example, to enable completions for bash, add to your shell profile:
    /// ```sh
    /// eval "$(fgm completions  --shell <YOUR_SHELL>)"
    /// ```
    /// where `<YOUR_SHELL>` is the shell you are using.
    Completions {
        #[clap(short, long, default_value = "bash")]
        shell: clap_complete::Shell,
    },
}

impl Subcommand {
    pub fn run(&self, ctx: &mut FgmContext) -> Result<()> {
        match self {
            Subcommand::Install { version, update } => {
                ctx.update = *update;
                install(ctx, version)?;
            }
            Subcommand::LsLocal { sort, reverse } => {
                list_installed(ctx, *sort, *reverse);
            }
            Subcommand::LsRemote {
                sort,
                reverse,
                update,
            } => {
                ctx.update = *update;
                list_remote(ctx, *sort, *reverse)?;
            }
            Subcommand::Uninstall { version } => {
                uninstall(ctx, version)?;
            }
            Subcommand::Use { version, update } => {
                ctx.update = *update;
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
            Subcommand::Completions { shell } => gen_completions(*shell)?,
        }
        Ok(())
    }
}
