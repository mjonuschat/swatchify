use std::path::Path;
use thiserror::Error;
use walkdir::WalkDir;

const PRINTABLE_FILE_TYPES: [&str; 4] = ["step", "3mf", "stl", "obj"];

#[derive(Debug, Error)]
pub(crate) enum PathError {
    #[error("Path `{0}` could not be resolved")]
    Canonicalize(#[from] std::io::Error),
    #[error("File or directory `{0}` is not accessible")]
    Inaccessible(String),
}

fn is_printable_file(file: &str) -> bool {
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

pub(crate) fn list_existing_swatches(path: &Path) -> Vec<String> {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(is_dir_or_printable)
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>()
}

pub(crate) fn create_output_dir(path: &Path) -> anyhow::Result<(), PathError> {
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
