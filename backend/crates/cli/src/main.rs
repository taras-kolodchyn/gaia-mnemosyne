use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "gaia-mnemosyne-cli")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show CLI version.
    Version,
    /// Run filesystem ingestion pipeline.
    Ingest,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => println!("Gaia Mnemosyne version 0.1.0"),
        Some(Commands::Ingest) => {
            if let Err(err) = mnemo_ingest::fs_ingest_runner::run_filesystem_ingestion(None).await {
                eprintln!("Ingestion failed: {err}");
            }
        }
        None => println!("Gaia Mnemosyne CLI initialized"),
    }
}
