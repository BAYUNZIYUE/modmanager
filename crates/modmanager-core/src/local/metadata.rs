use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::models::{ModProvider, ModVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub provider: ModProvider,
    pub mod_id: String,
    pub slug: String,
    pub name: String,
    pub installed_version_id: String,
    pub installed_version_number: String,
    pub filename: String,
    pub hash_sha1: Option<String>,
    pub hash_sha512: Option<String>,
    pub download_url: Option<String>,
}

impl ModMetadata {
    pub fn from_version(version: &ModVersion, mod_name: &str, provider: ModProvider) -> Self {
        Self {
            provider,
            mod_id: version.mod_id.clone(),
            slug: mod_name.to_lowercase().replace(' ', "-"),
            name: mod_name.to_string(),
            installed_version_id: version.id.clone(),
            installed_version_number: version.version_number.clone(),
            filename: version.filename.clone(),
            hash_sha1: version.hash_sha1.clone(),
            hash_sha512: version.hash_sha512.clone(),
            download_url: Some(version.download_url.to_string()),
        }
    }

    pub fn toml_path(mods_dir: &Path, filename: &str) -> PathBuf {
        let stem = filename.trim_end_matches(".jar");
        mods_dir.join(format!("{stem}.toml"))
    }

    pub fn save(&self, mods_dir: &Path) -> anyhow::Result<()> {
        let path = Self::toml_path(mods_dir, &self.filename);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metadata: Self = toml::from_str(&content)?;
        Ok(metadata)
    }
}
