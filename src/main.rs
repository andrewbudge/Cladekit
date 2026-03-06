use clap::{Parser, Subcommand};

// point to my command  
mod cmd;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "ghd")]
    Getheaders(cmd::getheaders::GetheadersArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Getheaders(args) => cmd::getheaders::run(args),
    }
}
