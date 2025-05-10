use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "schemr", author, version, about = "MySQL schema dump & diff CLI (Rust)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Configure database environments in schemr.toml
    Configure {},
    /// Dump schema of <env> into JSON files under the output directory
    DumpSchema {
        /// Environment name to dump (must exist in schemr.toml)
        #[arg(long)]
        env: String,
        /// Output directory for JSON files
        #[arg(short, long, default_value = "schemr-dumps")]
        output: String,
    },
    /// Compare schemas from two environments' dump directories
    Compare {
        #[arg(long)]
        env1: String,
        #[arg(long)]
        env2: String,
    },
}
