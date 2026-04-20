use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mod_dirs: Vec<String>,
    pub active_mod_dir: Option<String>,
    pub preferred_game_version: Option<String>,
    pub preferred_loader: Option<String>,
    #[serde(default)]
    pub use_mirror: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "zh-CN".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mod_dirs: vec![],
            active_mod_dir: default_minecraft_dir(),
            preferred_game_version: None,
            preferred_loader: None,
            use_mirror: false,
            language: default_language(),
        }
    }
}

fn default_minecraft_dir() -> Option<String> {
    let home = dirs::home_dir()?;
    let mc_dir = if cfg!(target_os = "windows") {
        home.join("AppData").join("Roaming").join(".minecraft")
    } else if cfg!(target_os = "macos") {
        home.join("Library")
            .join("Application Support")
            .join("minecraft")
    } else {
        home.join(".minecraft")
    };
    Some(mc_dir.to_string_lossy().to_string())
}

impl AppConfig {
    pub fn config_dir() -> anyhow::Result<PathBuf> {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("modmanager-rs");
        Ok(dir)
    }

    pub fn config_path() -> anyhow::Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn mods_dir(&self) -> Option<PathBuf> {
        self.active_mod_dir
            .as_ref()
            .map(|d| PathBuf::from(d).join("mods"))
    }
}
