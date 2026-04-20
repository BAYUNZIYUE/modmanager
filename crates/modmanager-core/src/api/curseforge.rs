use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::ModPlatform;
use crate::models::{
    DependencyType, GameVersion, ModAuthor, ModDependency, ModInfo, ModLoader, ModProvider,
    ModVersion, SearchFilter, SearchResult, Side, VersionType,
};

const CURSEFORGE_API_BASE: &str = "https://api.curseforge.com/v1";
const CURSEFORGE_API_KEY: &str = "$2a$10$wuAJuNZuted3NORVmpgUC.m8sI.pv1tOPKZyBgLFGjxFp/br0lZCC";
const MINECRAFT_GAME_ID: u32 = 432;
const MOD_CLASS_ID: u32 = 6;
const PAGE_SIZE: u32 = 25;

pub struct CurseForgeApi {
    client: Client,
}

impl CurseForgeApi {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT,
                    "application/json".parse().unwrap(),
                );
                headers.insert(
                    "x-api-key",
                    CURSEFORGE_API_KEY.parse().unwrap(),
                );
                headers
            })
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    fn loader_to_curseforge(loader: ModLoader) -> i32 {
        match loader {
            ModLoader::Forge => 1,
            ModLoader::Cauldron => 2,
            ModLoader::LiteLoader => 3,
            ModLoader::Fabric => 4,
            ModLoader::Quilt => 5,
            ModLoader::NeoForge => 6,
            _ => 0,
        }
    }
}

#[derive(Deserialize)]
struct CfResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct CfPagination {
    total_count: u32,
}

#[derive(Deserialize)]
struct CfSearchResult {
    pagination: CfPagination,
    data: Vec<CfMod>,
}

#[derive(Deserialize)]
struct CfMod {
    id: u32,
    name: String,
    slug: Option<String>,
    summary: Option<String>,
    authors: Option<Vec<CfAuthor>>,
    links: Option<CfLinks>,
    logo: Option<CfAsset>,
    download_count: Option<u64>,
    categories: Option<Vec<CfCategory>>,
    game_version_latest_files: Option<Vec<CfGameVersionFile>>,
}

#[derive(Deserialize)]
struct CfAuthor {
    name: String,
    url: Option<String>,
}

#[derive(Deserialize)]
struct CfLinks {
    website_url: Option<String>,
    source_url: Option<String>,
    issues_url: Option<String>,
}

#[derive(Deserialize)]
struct CfAsset {
    thumbnail_url: Option<String>,
}

#[derive(Deserialize)]
struct CfCategory {
    name: String,
}

#[derive(Deserialize)]
struct CfGameVersionFile {
    game_version: Option<String>,
    file_id: Option<u32>,
}

#[derive(Deserialize)]
struct CfFile {
    id: u32,
    mod_id: u32,
    display_name: String,
    file_name: String,
    file_length: u64,
    download_url: Option<String>,
    file_date: String,
    release_type: i32,
    game_versions: Option<Vec<String>>,
    dependencies: Option<Vec<CfDependency>>,
    hashes: Option<Vec<CfFileHash>>,
    sortable_game_versions: Option<Vec<CfSortableGameVersion>>,
}

#[derive(Deserialize)]
struct CfDependency {
    mod_id: u32,
    relation_type: i32,
}

#[derive(Deserialize)]
struct CfFileHash {
    value: String,
    algo: i32,
}

#[derive(Deserialize)]
struct CfSortableGameVersion {
    game_version_name: Option<String>,
    game_version_type: Option<u32>,
}

impl CfFile {
    fn to_mod_version(&self) -> Option<ModVersion> {
        let download_url = self.download_url.as_ref()?;
        let url = url::Url::parse(download_url).ok()?;

        let version_type = match self.release_type {
            1 => VersionType::Release,
            2 => VersionType::Beta,
            3 => VersionType::Alpha,
            _ => VersionType::Release,
        };

        let game_versions = self
            .game_versions
            .as_ref()
            .map(|vs| vs.iter().map(|v| GameVersion::new(v)).collect())
            .unwrap_or_default();

        let mut hash_sha1 = None;
        let mut hash_md5 = None;
        if let Some(hashes) = &self.hashes {
            for h in hashes {
                match h.algo {
                    1 => hash_sha1 = Some(h.value.clone()),
                    2 => hash_md5 = Some(h.value.clone()),
                    _ => {}
                }
            }
        }

        let loaders = self
            .sortable_game_versions
            .as_ref()
            .map(|sgvs| {
                sgvs.iter()
                    .filter(|sgv| sgv.game_version_type == Some(2))
                    .filter_map(|sgv| {
                        let name = sgv.game_version_name.as_deref()?;
                        match name.to_lowercase().as_str() {
                            "forge" => Some(ModLoader::Forge),
                            "neoforge" => Some(ModLoader::NeoForge),
                            "fabric" => Some(ModLoader::Fabric),
                            "quilt" => Some(ModLoader::Quilt),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let dependencies = self
            .dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|d| d.mod_id != 0)
                    .map(|d| ModDependency {
                        mod_id: d.mod_id.to_string(),
                        dependency_type: match d.relation_type {
                            1 => DependencyType::Required,
                            2 => DependencyType::Optional,
                            3 => DependencyType::Incompatible,
                            4 => DependencyType::Embedded,
                            _ => DependencyType::Required,
                        },
                        version_id: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(ModVersion {
            id: self.id.to_string(),
            mod_id: self.mod_id.to_string(),
            version_number: self.display_name.clone(),
            version_name: self.display_name.clone(),
            version_type,
            game_versions,
            loaders,
            download_url: url,
            filename: self.file_name.clone(),
            file_size: self.file_length,
            hash_sha1,
            hash_sha512: None,
            hash_md5,
            date: chrono::DateTime::parse_from_rfc3339(&self.file_date)
                .map(|dt| dt.to_utc())
                .unwrap_or(chrono::Utc::now()),
            changelog: None,
            side: Side::Universal,
            dependencies,
        })
    }
}

impl CfMod {
    fn to_mod_info(&self) -> ModInfo {
        let icon_url = self
            .logo
            .as_ref()
            .and_then(|l| l.thumbnail_url.clone())
            .and_then(|u| url::Url::parse(&u).ok());

        let website_url = self
            .links
            .as_ref()
            .and_then(|l| l.website_url.clone())
            .and_then(|u| url::Url::parse(&u).ok());

        let source_url = self
            .links
            .as_ref()
            .and_then(|l| l.source_url.clone())
            .and_then(|u| url::Url::parse(&u).ok());

        let issues_url = self
            .links
            .as_ref()
            .and_then(|l| l.issues_url.clone())
            .and_then(|u| url::Url::parse(&u).ok());

        ModInfo {
            id: self.id.to_string(),
            slug: self.slug.clone().unwrap_or_default(),
            name: self.name.clone(),
            description: self.summary.clone().unwrap_or_default(),
            provider: ModProvider::CurseForge,
            authors: self
                .authors
                .as_ref()
                .map(|a| {
                    a.iter()
                        .map(|author| ModAuthor {
                            name: author.name.clone(),
                            url: author.url.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            icon_url,
            website_url,
            source_url,
            issues_url,
            downloads: self.download_count.unwrap_or(0),
            categories: self
                .categories
                .as_ref()
                .map(|c| c.iter().map(|cat| cat.name.clone()).collect())
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl ModPlatform for CurseForgeApi {
    fn provider(&self) -> ModProvider {
        ModProvider::CurseForge
    }

    async fn search(&self, filter: &SearchFilter) -> anyhow::Result<SearchResult> {
        let mut params: Vec<(&str, String)> = vec![
            ("gameId", MINECRAFT_GAME_ID.to_string()),
            ("classId", MOD_CLASS_ID.to_string()),
            ("index", filter.offset.to_string()),
            ("pageSize", PAGE_SIZE.to_string()),
            ("sortOrder", "desc".to_string()),
        ];

        if let Some(ref query) = filter.query {
            params.push(("searchFilter", query.clone()));
        }

        if let Some(ref version) = filter.game_version {
            params.push(("gameVersion", version.to_string()));
        }

        if let Some(loader) = filter.loader {
            let cf_loader = Self::loader_to_curseforge(loader);
            if cf_loader != 0 {
                params.push(("modLoaderTypes", format!("[{cf_loader}]")));
            }
        }

        let url = format!("{}/mods/search", CURSEFORGE_API_BASE);
        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("CurseForge search request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<CfSearchResult> = resp
            .json()
            .await
            .context("Failed to parse CurseForge search response")?;

        let mods = result.data.data.into_iter().map(|m| m.to_mod_info()).collect();
        let total_count = result.data.pagination.total_count;

        Ok(SearchResult {
            mods,
            total_count,
            offset: filter.offset,
            limit: PAGE_SIZE,
        })
    }

    async fn get_mod_info(&self, mod_id: &str) -> anyhow::Result<ModInfo> {
        let url = format!("{}/mods/{mod_id}", CURSEFORGE_API_BASE);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("CurseForge get_mod_info request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<CfMod> = resp
            .json()
            .await
            .context("Failed to parse CurseForge mod info response")?;

        Ok(result.data.to_mod_info())
    }

    async fn get_mod_versions(&self, mod_id: &str) -> anyhow::Result<Vec<ModVersion>> {
        let url = format!("{}/mods/{mod_id}/files", CURSEFORGE_API_BASE);
        let resp = self
            .client
            .get(&url)
            .query(&[("pageSize", "10000")])
            .send()
            .await
            .context("CurseForge get_mod_versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<Vec<CfFile>> = resp
            .json()
            .await
            .context("Failed to parse CurseForge mod versions response")?;

        Ok(result
            .data
            .into_iter()
            .filter_map(|f| f.to_mod_version())
            .collect())
    }

    async fn get_version_by_id(&self, version_id: &str) -> anyhow::Result<ModVersion> {
        let url = format!("{}/mods/files/{version_id}", CURSEFORGE_API_BASE);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("CurseForge get_version_by_id request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<CfFile> = resp
            .json()
            .await
            .context("Failed to parse CurseForge version response")?;

        result
            .data
            .to_mod_version()
            .ok_or_else(|| anyhow::anyhow!("CurseForge file has no download URL"))
    }

    async fn get_multiple_versions(&self, version_ids: &[&str]) -> anyhow::Result<Vec<ModVersion>> {
        let url = format!("{}/mods/files", CURSEFORGE_API_BASE);

        let body = serde_json::json!({
            "fileIds": version_ids.iter().map(|id| id.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>()
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("CurseForge get_multiple_versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<Vec<CfFile>> = resp
            .json()
            .await
            .context("Failed to parse CurseForge multiple versions response")?;

        Ok(result
            .data
            .into_iter()
            .filter_map(|f| f.to_mod_version())
            .collect())
    }

    async fn get_version_changelog(&self, mod_id: &str, version_id: &str) -> anyhow::Result<String> {
        let url = format!("{}/mods/{mod_id}/files/{version_id}/changelog", CURSEFORGE_API_BASE);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("CurseForge get_version_changelog request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("CurseForge API error: {status} - {body}");
        }

        let result: CfResponse<String> = resp
            .json()
            .await
            .context("Failed to parse CurseForge changelog response")?;

        Ok(result.data)
    }
}
