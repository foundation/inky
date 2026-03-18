use serde::Deserialize;
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "inky.config.json";

#[derive(Debug, Deserialize, Default)]
pub struct ProjectConfig {
    /// Source directory (maps to input)
    pub src: Option<String>,
    /// Output directory (maps to output)
    pub dist: Option<String>,
    /// Number of grid columns
    pub columns: Option<u32>,
    /// Custom components directory (default: "components")
    pub components: Option<String>,
    /// Path to a JSON file with template merge data
    pub data: Option<String>,
    /// Use hybrid output mode (div + MSO ghost tables)
    pub hybrid: Option<bool>,
}

/// Search for `inky.config.json` starting from `start_dir` and walking up to the filesystem root.
/// Returns the parsed config and the directory it was found in.
pub fn load_config(start_dir: &Path) -> Option<(ProjectConfig, PathBuf)> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(CONFIG_FILENAME);
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate).ok()?;
            let config: ProjectConfig = serde_json::from_str(&content).ok()?;
            return Some((config, dir));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
