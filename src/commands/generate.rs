use crate::GeneratorOptions;
use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use maplit::hashmap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use walkdir::WalkDir;

const SWATCH_PARAMETERS: &[u8] = include_bytes!("../../templates/parameters.json");
const PRINTABLE_FILE_TYPES: [&str; 4] = ["step", "3mf", "stl", "obj"];

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Path `{0}` could not be resolved")]
    Canonicalize(#[from] std::io::Error),
    #[error("File or directory `{0}` is not accessible")]
    Inaccessible(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FilamentSwatchOptions {
    #[serde(rename = "$fn")]
    func: String,
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

pub fn is_printable_file(file: &str) -> bool {
    let extension = Path::new(&file.to_lowercase())
        .extension()
        .and_then(|v| v.to_str().map(|v| v.to_owned()));

    match extension {
        Some(ext) => PRINTABLE_FILE_TYPES.contains(&ext.as_str()),
        None => false,
    }
}

fn is_dir_or_printable(entry: &walkdir::DirEntry) -> bool {
    if entry.path().is_dir() {
        return true;
    }

    entry
        .file_name()
        .to_str()
        .map(is_printable_file)
        .unwrap_or(false)
}

fn read_existing_swatches(path: &Path) -> Vec<String> {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(is_dir_or_printable)
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>()
}

pub fn create_output_dir(path: &Path) -> Result<(), PathError> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(())
            } else {
                Err(PathError::Inaccessible(path.to_string_lossy().to_string()))
            }
        }
        Err(_e) => Ok(std::fs::create_dir_all(path)?),
    }
}

fn render(filament: &FilamentRecord, destination_folder: &Path) -> Result<()> {
    let defaults: FilamentSwatchOptions = serde_json::from_slice(SWATCH_PARAMETERS)?;
    let filename = PathBuf::from(filament.to_string()).with_extension(".3mf");

    let dst = destination_folder
        .join(&filament.material)
        .join(&filament.manufacturer);

    create_output_dir(&dst)?;

    let dst = dst.join(filename);
    let work_dir = tempfile::tempdir()?;

    let options = CustomizerSettings {
        parameter_sets: hashmap! {
            "Generator".to_string() => FilamentSwatchOptions {
                textstring1: format!("0.2mm @ {}°C", &filament.temperature),
                textstring2: filament.manufacturer.to_owned(),
                textstring3: filament.color.to_owned(),
                texttop_configurable: filament
                    .material
                    .chars()
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                ..defaults
            }
        },
        ..Default::default()
    };

    let customizer_path = work_dir.path().join("customizer.json");
    serde_json::to_writer_pretty(&File::create(&customizer_path)?, &options)?;

    Command::new("/Applications/OpenSCAD.app/Contents/MacOS/OpenSCAD")
        .arg("--export-format")
        .arg("binstl")
        .arg("-o")
        .arg(dst)
        .arg("-p")
        .arg(customizer_path)
        .arg("-P")
        .arg("Generator")
        .arg("/Users/mjonuschat/Downloads/Configurable_Filament_Swatch_VZC.scad")
        .output()?;

    Ok(())
}

pub(crate) fn write(options: &GeneratorOptions) -> Result<()> {
    let destination_folder = options
        .destination
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    create_output_dir(&destination_folder)?;
    let existing = read_existing_swatches(&destination_folder);

    let mut reader = csv::Reader::from_path(
        options
            .inventory
            .clone()
            .unwrap_or_else(|| PathBuf::from("./inventory.txt")),
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
        .try_for_each(|filament| render(filament, &destination_folder))?;
    Ok(())
}
