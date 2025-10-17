use std::fs;
use std::path::{Path, PathBuf};

pub fn get_stq_files(dir: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let paths = fs::read_dir(dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.display().to_string().ends_with(".stq") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    Ok(paths)
}
