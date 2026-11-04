use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub selected_workspace: Option<i64>,
    pub screenshot_dir: Option<PathBuf>,
    pub keybind_enabled: bool,
    pub keybind: Option<String>,
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Cannot determine XDG config directory")?
        .join("cybershot");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.toml"))
}
pub fn load() -> Result<Config> {
    let p = config_path()?;
    if !p.exists() {
        return Ok(Config::default());
    }
    Ok(toml::from_str(&fs::read_to_string(p)?)?)
}
pub fn save(cfg: &Config) -> Result<()> {
    fs::write(config_path()?, toml::to_string_pretty(cfg)?)?;
    Ok(())
}
pub fn screenshot_dir() -> Result<PathBuf> {
    let cfg = load()?;
    let path = cfg.screenshot_dir.unwrap_or_else(|| {
        dirs::picture_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("CyberShot")
    });
    fs::create_dir_all(&path)?;
    Ok(path)
}
