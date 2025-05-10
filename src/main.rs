use anyhow::Result;
mod cli;
use clap::Parser;
mod config;
mod schema;
mod compare;

use cli::{Cli, Commands};
use config::Config;

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Configure {} => {
            Config::configure()?;
        }
        Commands::DumpSchema { env, output } => {
            schema::dump_schema(&env, &output)?;
        }
        Commands::Compare { env1, env2 } => {
            compare::compare(&env1, &env2)?;
        }
    }
    Ok(())
}
