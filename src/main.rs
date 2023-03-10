use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser, ValueHint};
use commands::generate::{OutputFormat, SwatchOptions};
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
    /// Output format
    #[clap(long, default_value = "stl")]
    output_format: OutputFormat,
    /// Output format
    #[clap(long, value_hint=ValueHint::FilePath, default_value = commands::generate::OPEN_SCAD_PATH)]
    openscad_path: PathBuf,
    /// Force export and regenerate all existing files
    #[clap(short, long)]
    pub(crate) force: bool,
    /// Testing
    #[clap(flatten)]
    swatch_design: SwatchOptions,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Generate(options) => commands::generate::write(options)?,
    }
    Ok(())
}
