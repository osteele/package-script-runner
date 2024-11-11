mod cli;
mod config;
mod execution;
mod package_managers;
mod types;
mod themes;
mod tui;

#[cfg(test)]
pub mod tests {
    pub mod project_dir_mocks;
}

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
