use clap::Parser;
use fgm::{
    cli,
    config::{count_config_path, FgmContext},
};

fn init_context() -> FgmContext {
    let mut ctx = FgmContext::default();
    let config_path = count_config_path().unwrap_or_else(|e| {
        eprintln!("failed to get config path: {}", e);
        std::process::exit(1);
    });
    let config = fgm::config::FgmConfig::load(&config_path).unwrap_or_default();
    ctx.update_from_config(&config);

    ctx
}

fn main() {
    let ctx = init_context();
    let cli = cli::Cli::parse();
    if let Err(e) = cli.sub.run(&ctx) {
        println!("{:?}", e);
    }
}
