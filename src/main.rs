use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser, ValueHint};
use std::path::PathBuf;

mod commands;
mod helpers;

#[derive(clap::Parser, Debug)]
#[clap(author, about, version, name = "Customizable Filament Swatch Generator", color=ColorChoice::Auto)]
pub(crate) struct Cli {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action=ArgAction::Count)]
    pub verbose: u8,
    /// Hierarchical output directory structure
    #[clap(short, long, default_value_t = true)]
    pub organize: bool,
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    Generate(GeneratorOptions),
}

#[derive(Debug, clap::Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub(crate) struct GeneratorOptions {
    /// Inventory CSV
    #[clap(short, long, value_hint=ValueHint::FilePath)]
    inventory: Option<PathBuf>,
    /// Output directory
    #[clap(short, long, value_hint=ValueHint::DirPath)]
    destination: Option<PathBuf>,
    /// Force export and regenerate all existing files
    #[clap(short, long)]
    pub(crate) force: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Generate(options) => commands::generate::write(options)?,
    }
    Ok(())
}
