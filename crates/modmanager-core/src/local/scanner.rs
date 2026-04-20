use std::path::{Path, PathBuf};

use crate::local::ModMetadata;

#[derive(Debug, Clone)]
pub struct LocalMod {
    pub path: PathBuf,
    pub filename: String,
    pub enabled: bool,
    pub file_size: u64,
    pub metadata: Option<ModMetadata>,
}

impl LocalMod {
    pub fn name(&self) -> &str {
        &self.filename
    }

    pub fn display_name(&self) -> &str {
        self.metadata
            .as_ref()
            .map(|m| m.name.as_str())
            .unwrap_or_else(|| {
                self.filename
                    .trim_end_matches(".jar")
                    .trim_end_matches(".disabled")
            })
    }

    pub fn has_update_available(&self, latest_version_id: &str) -> bool {
        self.metadata
            .as_ref()
            .map(|m| m.installed_version_id != latest_version_id)
            .unwrap_or(true)
    }
}

pub struct ModScanner;

impl ModScanner {
    pub fn scan_mods_dir(mods_dir: &Path) -> anyhow::Result<Vec<LocalMod>> {
        if !mods_dir.exists() {
            std::fs::create_dir_all(mods_dir)?;
            return Ok(vec![]);
        }

        let mut mods = Vec::new();

        for entry in std::fs::read_dir(mods_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let enabled = filename.ends_with(".jar");
            let is_mod = enabled || filename.ends_with(".jar.disabled");

            if !is_mod {
                continue;
            }

            let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            let metadata = if enabled {
                let meta_path = ModMetadata::toml_path(mods_dir, &filename);
                if meta_path.exists() {
                    ModMetadata::load(&meta_path).ok()
                } else {
                    None
                }
            } else {
                None
            };

            mods.push(LocalMod {
                path,
                filename,
                enabled,
                file_size,
                metadata,
            });
        }

        mods.sort_by(|a, b| a.display_name().to_lowercase().cmp(&b.display_name().to_lowercase()));

        Ok(mods)
    }

    pub fn toggle_mod(local_mod: &LocalMod) -> anyhow::Result<PathBuf> {
        let new_path = if local_mod.enabled {
            local_mod.path.with_extension("jar.disabled")
        } else {
            let without_disabled = local_mod
                .path
                .to_str()
                .and_then(|s| s.strip_suffix(".disabled"))
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| local_mod.path.clone());
            without_disabled
        };

        std::fs::rename(&local_mod.path, &new_path)?;
        Ok(new_path)
    }

    pub fn delete_mod(local_mod: &LocalMod) -> anyhow::Result<()> {
        std::fs::remove_file(&local_mod.path)?;

        if let Some(ref _meta) = local_mod.metadata {
            let meta_path = ModMetadata::toml_path(
                local_mod.path.parent().unwrap_or(Path::new(".")),
                &local_mod.filename,
            );
            if meta_path.exists() {
                std::fs::remove_file(meta_path)?;
            }
        }

        Ok(())
    }
}
