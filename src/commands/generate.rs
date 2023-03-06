use crate::helpers::fs;
use crate::GeneratorOptions;
use anyhow::Result;
use clap::builder::PossibleValue;
use clap::ValueEnum;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use maplit::hashmap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

// Source: https://www.printables.com/model/27814-filament-swatch
const SWATCH_SCAD_FILE: &[u8] = include_bytes!("../../templates/swatch.scad");
const SWATCH_PARAMETERS: &[u8] = include_bytes!("../../templates/parameters.json");

#[cfg(target_os = "macos")]
pub(crate) const OPEN_SCAD_PATH: &str = "/Applications/OpenSCAD.app/Contents/MacOS/OpenSCAD";

#[cfg(windows)]
pub(crate) const OPEN_SCAD_PATH: &str = r#"C:\Program Files\Openscad\openscad.exe"#;

// Unix like OS - assume openscad binary is in the path
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) const OPEN_SCAD_PATH: &str = r#"openscad"#;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FilamentSwatchOptions {
    #[serde(rename = "$fn")]
    fragments: String,
    edge_tests: String,
    font_recessed: String,
    fontname: String,
    h: String,
    linesep: String,
    r_hole: String,
    r_indent: String,
    round: String,
    step_thickness_correction: String,
    steps_text: String,
    steps_text_format: String,
    steps_text_rotate: String,
    steps_textheight: String,
    steps_textsize: String,
    steps_thickness: String,
    tack_hole: String,
    test_circles: String,
    text_type: String,
    textsize_lower: String,
    textsize_upper: String,
    textstring1: String,
    textstring2: String,
    textstring3: String,
    texttop: String,
    texttop_configurable: String,
    th: String,
    thole_d: String,
    thole_top_shiftright: String,
    w: String,
    wall: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CustomizerSettings {
    #[serde(rename = "parameterSets")]
    parameter_sets: HashMap<String, FilamentSwatchOptions>,
    #[serde(rename = "fileFormatVersion")]
    file_format_version: String,
}

impl Default for CustomizerSettings {
    fn default() -> Self {
        Self {
            parameter_sets: HashMap::new(),
            file_format_version: "1".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FilamentRecord {
    manufacturer: String,
    color: String,
    material: String,
    temperature: i32,
}

impl Display for FilamentRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {}",
            &self.manufacturer, &self.material, &self.color
        )
    }
}

fn render_text_field(filament: &FilamentRecord, field: &OutputField) -> String {
    match field {
        OutputField::Manufacturer => filament.manufacturer.to_string(),
        OutputField::Color => filament.color.to_string(),
        OutputField::Temperature => format!("0.2mm @ {}°C", filament.temperature),
        OutputField::Material => filament.material.to_string(),
    }
}

fn render(
    filament: &FilamentRecord,
    destination_folder: &Path,
    options: &GeneratorOptions,
) -> Result<()> {
    let defaults: FilamentSwatchOptions = serde_json::from_slice(SWATCH_PARAMETERS)?;
    let filename = match options.output_format {
        OutputFormat::ThreeMf => PathBuf::from(filament.to_string()).with_extension("3mf"),
        OutputFormat::Stl => PathBuf::from(filament.to_string()).with_extension("stl"),
    };

    let dst = destination_folder
        .join(&filament.material)
        .join(&filament.manufacturer);

    fs::create_output_dir(&dst)?;

    let dst = dst.join(filename);
    let work_dir = tempfile::tempdir()?;

    let swatch_options = FilamentSwatchOptions {
        textstring1: render_text_field(filament, &options.swatch_design.text_upper),
        textstring2: render_text_field(filament, &options.swatch_design.text_lower_left),
        textstring3: render_text_field(filament, &options.swatch_design.text_lower_right),
        texttop_configurable: filament
            .material
            .chars()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(" "),
        textsize_lower: options.swatch_design.text_size_lower.to_string(),
        textsize_upper: options.swatch_design.text_size_upper.to_string(),
        w: options.swatch_design.width.to_string(),
        h: options.swatch_design.height.to_string(),
        ..defaults
    };

    let settings = CustomizerSettings {
        parameter_sets: hashmap! {
            "Generator".to_string() => swatch_options
        },
        ..Default::default()
    };

    let swatch_path = work_dir.path().join("swatch.scad");
    std::fs::write(&swatch_path, SWATCH_SCAD_FILE)?;

    let swatch_parameters = work_dir.path().join("settings.json");
    serde_json::to_writer_pretty(&File::create(&swatch_parameters)?, &settings)?;

    Command::new(&options.openscad_path)
        .arg("-o")
        .arg(dst)
        .arg("-p")
        .arg(swatch_parameters)
        .arg("-P")
        .arg("Generator")
        .arg(swatch_path)
        .output()?;

    Ok(())
}

pub(crate) fn write(options: &GeneratorOptions) -> Result<()> {
    let destination_folder = options
        .destination
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    fs::create_output_dir(&destination_folder)?;
    let existing = fs::list_existing_swatches(&destination_folder);

    let mut reader = csv::Reader::from_path(
        options
            .inventory
            .clone()
            .unwrap_or_else(|| PathBuf::from("inventory.txt")),
    )?;

    let filaments: Vec<_> = reader
        .deserialize()
        .filter_map(Result::ok)
        .filter(|f: &FilamentRecord| !existing.contains(&f.to_string()))
        .collect();

    let progress_bar_style =
        ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40} {pos:>7}/{len:7}")?;

    filaments
        .par_iter()
        .progress_with_style(progress_bar_style)
        .try_for_each(|filament| render(filament, &destination_folder, options))?;
    Ok(())
}

#[derive(clap::ValueEnum, Serialize, Deserialize, Clone, Debug)]
pub(crate) enum OutputField {
    Manufacturer,
    Color,
    Temperature,
    Material,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum OutputFormat {
    ThreeMf,
    Stl,
}

impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Stl, Self::ThreeMf]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Stl => PossibleValue::new("stl").help("Generate .stl files"),
            Self::ThreeMf => PossibleValue::new("3mf").help("Generate .3mf files"),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, clap::Args)]
pub(crate) struct SwatchOptions {
    /// Width of the filament swatch
    #[clap(long, default_value_t = 80.0)]
    pub width: f32,
    /// Height of the filament swatch
    #[clap(long, default_value_t = 32.0)]
    pub height: f32,
    /// Upper text line
    #[clap(long, default_value = "temperature")]
    pub text_upper: OutputField,
    /// Left lower text line
    #[clap(long, default_value = "manufacturer")]
    pub text_lower_left: OutputField,
    /// Left lower text line
    #[clap(long, default_value = "color")]
    pub text_lower_right: OutputField,
    /// Upper text size in mm
    #[clap(long, default_value_t = 4)]
    pub text_size_upper: i8,
    /// Lower text size in mm
    #[clap(long, default_value_t = 5)]
    pub text_size_lower: i8,
}
