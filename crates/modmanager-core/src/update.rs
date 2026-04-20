use std::path::Path;

use crate::api::ModPlatform;
use crate::local::{LocalMod, ModMetadata};
use crate::local::scanner::ModScanner;
use crate::models::{GameVersion, ModLoader, ModProvider};

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub local_mod: LocalMod,
    pub latest_version_id: String,
    pub latest_version_number: String,
    pub latest_filename: String,
    pub latest_download_url: String,
    pub latest_date: String,
    pub provider: ModProvider,
}

pub struct UpdateChecker<'a> {
    pub curseforge: &'a dyn ModPlatform,
    pub modrinth: &'a dyn ModPlatform,
    pub game_version: Option<GameVersion>,
    pub loader: Option<ModLoader>,
}

impl<'a> UpdateChecker<'a> {
    pub fn new(
        curseforge: &'a dyn ModPlatform,
        modrinth: &'a dyn ModPlatform,
    ) -> Self {
        Self {
            curseforge,
            modrinth,
            game_version: None,
            loader: None,
        }
    }

    pub fn with_game_version(mut self, version: Option<GameVersion>) -> Self {
        self.game_version = version;
        self
    }

    pub fn with_loader(mut self, loader: Option<ModLoader>) -> Self {
        self.loader = loader;
        self
    }

    pub async fn check_updates(
        &self,
        mods_dir: &Path,
    ) -> anyhow::Result<Vec<UpdateInfo>> {
        let local_mods = ModScanner::scan_mods_dir(mods_dir)?;
        let mut updates = Vec::new();

        for local_mod in &local_mods {
            if !local_mod.enabled {
                continue;
            }

            let Some(ref meta) = local_mod.metadata else {
                continue;
            };

            let platform = match meta.provider {
                ModProvider::CurseForge => self.curseforge,
                ModProvider::Modrinth => self.modrinth,
                ModProvider::Bmclapi => self.curseforge,
            };

            let versions = match platform.get_mod_versions(&meta.mod_id).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch versions for {}: {e}",
                        meta.name
                    );
                    continue;
                }
            };

            let compatible_version = versions
                .iter()
                .filter(|v| {
                    if let Some(ref gv) = self.game_version {
                        if !v.game_versions.iter().any(|v_gv| v_gv.as_str() == gv.as_str()) {
                            return false;
                        }
                    }
                    if let Some(loader) = self.loader {
                        if !v.loaders.contains(&loader) {
                            return false;
                        }
                    }
                    true
                })
                .max_by(|a, b| a.date.cmp(&b.date));

            if let Some(latest) = compatible_version {
                if local_mod.has_update_available(&latest.id) {
                    updates.push(UpdateInfo {
                        local_mod: local_mod.clone(),
                        latest_version_id: latest.id.clone(),
                        latest_version_number: latest.version_number.clone(),
                        latest_filename: latest.filename.clone(),
                        latest_download_url: latest.download_url.to_string(),
                        latest_date: latest.date.to_rfc3339(),
                        provider: meta.provider,
                    });
                }
            }
        }

        Ok(updates)
    }
}
