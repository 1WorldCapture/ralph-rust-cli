use clap::Parser;

/// Ralph CLI - A dispatcher for AI provider agents
#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Display version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("ralph {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            // No subcommand provided, show help
            println!("ralph {} - A dispatcher for AI provider agents", env!("CARGO_PKG_VERSION"));
            println!();
            println!("Use 'ralph --help' for more information.");
        }
    }
}
