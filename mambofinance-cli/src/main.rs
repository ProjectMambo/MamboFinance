use clap::{Parser, Subcommand};
use mambofinance_lib::user::User; 

#[derive(Parser)]
#[command(name = "mambo")]
#[command(about = "Mambo Finance Tracker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new transaction
    Add {
        amount: f64,
        category: String,
    },
}

fn main() {
    // let cli = Cli::parse();

    // match &cli.command {
    //     Commands::Add { amount, category } => {
    //         println!("Logging ${amount} under {category}...");
    //         // Call your library code here:
    //         // mambofinance_lib::add_transaction(amount, category);
    //     }
    // }
}