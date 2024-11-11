mod cli;
mod commands;
mod config;
mod execution;
mod package_managers;
mod project;
mod script_type;
mod themes;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
