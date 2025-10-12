use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub last_keymap_path: Option<String>,
}

pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("qmk_viewer");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;

    Ok(config_dir)
}

pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.json"))
}

pub fn load_app_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

pub fn save_app_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

pub fn save_keymap_file(source_path: &str) -> Result<String> {
    let config_dir = get_config_dir()?;
    let content = fs::read_to_string(source_path)?;

    // Determine file extension
    let source_path = Path::new(source_path);
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("json");

    let saved_filename = format!("last_keymap.{}", extension);
    let saved_path = config_dir.join(&saved_filename);

    // Write the content to the saved file
    fs::write(&saved_path, content)?;

    // Return the path as string
    Ok(saved_path.to_string_lossy().to_string())
}

pub fn clear_saved_keymap() -> Result<()> {
    let config_dir = get_config_dir()?;

    // Remove any saved keymap files
    let keymap_files = ["last_keymap.json", "last_keymap.c", "last_keymap.h"];
    for filename in &keymap_files {
        let path = config_dir.join(filename);
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    // Clear the config
    let mut config = load_app_config()?;
    config.last_keymap_path = None;
    save_app_config(&config)?;

    Ok(())
}

pub fn get_saved_keymap_path() -> Result<Option<String>> {
    let config_dir = get_config_dir()?;

    // Check for saved keymap files in order of preference
    let keymap_files = ["last_keymap.json", "last_keymap.c", "last_keymap.h"];
    for filename in &keymap_files {
        let path = config_dir.join(filename);
        if path.exists() {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}
