use anyhow::Context;
use reqwest::Client;

use crate::models::GameVersion;

const BMCLAPI_MIRROR: &str = "https://bmclapi2.bangbang93.com";
const CURSEFORGE_PROXY: &str = "https://bmclapi2.bangbang93.com/curseforge";

pub struct BmclapiMirror {
    client: Client,
}

impl BmclapiMirror {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create BMCLAPI HTTP client")?;
        Ok(Self { client })
    }

    pub fn rewrite_download_url(original_url: &str) -> Option<String> {
        if original_url.contains("edge.forgecdn.net") || original_url.contains("mediafilez.forgecdn.net") || original_url.contains("curseforge.com") {
            let path = original_url
                .split(".net/")
                .nth(1)
                .or_else(|| original_url.split(".com/").nth(1))?;
            Some(format!("{CURSEFORGE_PROXY}/{}", path))
        } else if original_url.contains("cdn.modrinth.com") || original_url.contains("github.com") {
            let path = if original_url.contains("cdn.modrinth.com") {
                original_url.replace("https://cdn.modrinth.com", &format!("{BMCLAPI_MIRROR}/modrinth"))
            } else {
                return None;
            };
            Some(path)
        } else {
            None
        }
    }

    pub async fn get_curseforge_versions(
        &self,
        mod_id: &str,
        game_version: Option<&GameVersion>,
        loader: Option<crate::models::ModLoader>,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/mods/{mod_id}/files", CURSEFORGE_PROXY);

        let mut params = vec![("pageSize", "10000".to_string())];
        if let Some(v) = game_version {
            params.push(("gameVersion", v.to_string()));
        }
        if let Some(l) = loader {
            let cf_id = l.curseforge_id();
            if cf_id != 0 {
                params.push(("modLoaderType", cf_id.to_string()));
            }
        }

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("BMCLAPI CurseForge versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("BMCLAPI error: {status} - {body}");
        }

        resp.json().await.context("Failed to parse BMCLAPI response")
    }

    pub async fn get_modrinth_versions(
        &self,
        mod_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/modrinth/v2/project/{mod_id}/version", BMCLAPI_MIRROR);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("BMCLAPI Modrinth versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("BMCLAPI error: {status} - {body}");
        }

        resp.json().await.context("Failed to parse BMCLAPI response")
    }

    pub async fn get_game_versions(&self) -> anyhow::Result<Vec<GameVersion>> {
        let url = format!("{}/mc/game/version", BMCLAPI_MIRROR);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("BMCLAPI game versions request failed")?;

        if !resp.status().is_success() {
            anyhow::bail!("BMCLAPI error: {}", resp.status());
        }

        let versions: serde_json::Value = resp.json().await?;
        let versions = versions
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("id")?.as_str().map(GameVersion::new))
                    .collect()
            })
            .unwrap_or_default();

        Ok(versions)
    }
}
