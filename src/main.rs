use clap::Parser;
use fgm::{cli, config::FgmConfig};

fn main() {
    let config = FgmConfig {
        installations_dir: "/usr/local/fgm".to_string(),
        gate_path: "/usr/local/go".to_string(),
        remote_source: "https://go.dev/dl/".to_string(),
    };
    let cli = cli::Cli::parse();
    if let Err(e) = cli.sub.run(&config) {
        println!("{:?}", e);
    }
}
