use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;

use super::ModPlatform;
use crate::models::{
    DependencyType, GameVersion, ModAuthor, ModDependency, ModInfo, ModLoader, ModProvider,
    ModVersion, SearchFilter, SearchResult, Side, VersionType,
};

const MODRINTH_API_BASE: &str = "https://api.modrinth.com/v2";

pub struct ModrinthApi {
    client: Client,
    user_agent: String,
}

impl ModrinthApi {
    pub fn new() -> anyhow::Result<Self> {
        Self::with_user_agent("modmanager-rs/0.1.0 (github.com/BAYUNZIYUE/modmanager)")
    }

    pub fn with_user_agent(user_agent: &str) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
                headers
            })
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            user_agent: user_agent.to_string(),
        })
    }
}

#[derive(serde::Deserialize)]
struct MrSearchResult {
    hits: Vec<MrProject>,
    total_hits: u32,
    offset: u32,
    limit: u32,
}

#[derive(serde::Deserialize)]
struct MrProject {
    project_id: String,
    slug: Option<String>,
    title: String,
    description: Option<String>,
    author: Option<String>,
    icon_url: Option<String>,
    page_url: Option<String>,
    source_url: Option<String>,
    issues_url: Option<String>,
    downloads: Option<u64>,
    categories: Option<Vec<String>>,
    project_type: Option<String>,
}

#[derive(serde::Deserialize)]
struct MrVersion {
    id: String,
    project_id: String,
    version_number: String,
    name: String,
    version_type: Option<String>,
    game_versions: Option<Vec<String>>,
    loaders: Option<Vec<String>>,
    files: Option<Vec<MrFile>>,
    date_published: String,
    changelog: Option<String>,
    dependencies: Option<Vec<MrDependency>>,
    side_types: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct MrFile {
    hashes: Option<HashMap<String, String>>,
    url: String,
    filename: String,
    size: Option<u64>,
    primary: Option<bool>,
}

#[derive(serde::Deserialize)]
struct MrDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: Option<String>,
}

impl MrProject {
    fn to_mod_info(&self) -> ModInfo {
        let icon_url = self
            .icon_url
            .as_ref()
            .and_then(|u| url::Url::parse(u).ok());

        let website_url = self
            .page_url
            .as_ref()
            .and_then(|u| url::Url::parse(u).ok());

        let source_url = self
            .source_url
            .as_ref()
            .and_then(|u| url::Url::parse(u).ok());

        let issues_url = self
            .issues_url
            .as_ref()
            .and_then(|u| url::Url::parse(u).ok());

        let authors = self
            .author
            .as_ref()
            .map(|a| {
                vec![ModAuthor {
                    name: a.clone(),
                    url: self.page_url.clone(),
                }]
            })
            .unwrap_or_default();

        ModInfo {
            id: self.project_id.clone(),
            slug: self.slug.clone().unwrap_or_default(),
            name: self.title.clone(),
            description: self.description.clone().unwrap_or_default(),
            provider: ModProvider::Modrinth,
            authors,
            icon_url,
            website_url,
            source_url,
            issues_url,
            downloads: self.downloads.unwrap_or(0),
            categories: self.categories.clone().unwrap_or_default(),
        }
    }
}

impl MrVersion {
    fn to_mod_version(&self) -> Option<ModVersion> {
        let primary_file = self
            .files
            .as_ref()?
            .iter()
            .find(|f| f.primary == Some(true))
            .or_else(|| self.files.as_ref()?.first())?;

        let download_url = url::Url::parse(&primary_file.url).ok()?;

        let version_type = self
            .version_type
            .as_deref()
            .map(VersionType::from_str)
            .unwrap_or(VersionType::Release);

        let game_versions = self
            .game_versions
            .as_ref()
            .map(|vs| vs.iter().map(|v| GameVersion::new(v)).collect())
            .unwrap_or_default();

        let loaders = self
            .loaders
            .as_ref()
            .map(|ls| {
                ls.iter()
                    .filter_map(|l| match l.to_lowercase().as_str() {
                        "forge" => Some(ModLoader::Forge),
                        "neoforge" => Some(ModLoader::NeoForge),
                        "fabric" => Some(ModLoader::Fabric),
                        "quilt" => Some(ModLoader::Quilt),
                        "liteloader" => Some(ModLoader::LiteLoader),
                        "babric" => Some(ModLoader::Babric),
                        "bta" => Some(ModLoader::BTA),
                        "legacyfabric" => Some(ModLoader::LegacyFabric),
                        "ornithe" => Some(ModLoader::Ornithe),
                        "rift" => Some(ModLoader::Rift),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut hash_sha1 = None;
        let mut hash_sha512 = None;
        if let Some(hashes) = &primary_file.hashes {
            hash_sha1 = hashes.get("sha1").cloned();
            hash_sha512 = hashes.get("sha512").cloned();
        }

        let side = self
            .side_types
            .as_ref()
            .and_then(|sides| {
                let client = sides.iter().any(|s| s == "client");
                let server = sides.iter().any(|s| s == "server");
                match (client, server) {
                    (true, true) => Some(Side::Universal),
                    (true, false) => Some(Side::Client),
                    (false, true) => Some(Side::Server),
                    _ => Some(Side::Unknown),
                }
            })
            .unwrap_or(Side::Unknown);

        let dependencies = self
            .dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|d| d.project_id.is_some())
                    .map(|d| ModDependency {
                        mod_id: d.project_id.clone().unwrap(),
                        dependency_type: match d.dependency_type.as_deref() {
                            Some("required") => DependencyType::Required,
                            Some("optional") => DependencyType::Optional,
                            Some("incompatible") => DependencyType::Incompatible,
                            Some("embedded") => DependencyType::Embedded,
                            _ => DependencyType::Required,
                        },
                        version_id: d.version_id.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(ModVersion {
            id: self.id.clone(),
            mod_id: self.project_id.clone(),
            version_number: self.version_number.clone(),
            version_name: self.name.clone(),
            version_type,
            game_versions,
            loaders,
            download_url,
            filename: primary_file.filename.clone(),
            file_size: primary_file.size.unwrap_or(0),
            hash_sha1,
            hash_sha512,
            hash_md5: None,
            date: chrono::DateTime::parse_from_rfc3339(&self.date_published)
                .map(|dt| dt.to_utc())
                .unwrap_or(chrono::Utc::now()),
            changelog: self.changelog.clone(),
            side,
            dependencies,
        })
    }
}

use std::collections::HashMap;

#[async_trait]
impl ModPlatform for ModrinthApi {
    fn provider(&self) -> ModProvider {
        ModProvider::Modrinth
    }

    async fn search(&self, filter: &SearchFilter) -> anyhow::Result<SearchResult> {
        let mut facets: Vec<Vec<String>> = vec![];
        facets.push(vec!["project_type:mod".to_string()]);

        if let Some(ref version) = filter.game_version {
            facets.push(vec![format!("versions:{}", version)]);
        }

        if let Some(loader) = filter.loader {
            facets.push(vec![format!("categories:{}", loader.modrinth_name())]);
        }

        if let Some(ref category) = filter.category {
            facets.push(vec![format!("categories:{category}")]);
        }

        let facets_json = serde_json::to_string(&facets)?;

        let mut params: Vec<(&str, String)> = vec![
            ("facets", facets_json),
            ("offset", filter.offset.to_string()),
            ("limit", filter.limit.to_string()),
        ];

        if let Some(ref query) = filter.query {
            params.push(("query", query.clone()));
        }

        let resp = self
            .client
            .get(format!("{}/search", MODRINTH_API_BASE))
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .query(&params)
            .send()
            .await
            .context("Modrinth search request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        let result: MrSearchResult = resp
            .json()
            .await
            .context("Failed to parse Modrinth search response")?;

        let mods = result.hits.into_iter().map(|p| p.to_mod_info()).collect();

        Ok(SearchResult {
            mods,
            total_count: result.total_hits,
            offset: result.offset,
            limit: result.limit,
        })
    }

    async fn get_mod_info(&self, mod_id: &str) -> anyhow::Result<ModInfo> {
        let url = format!("{}/project/{mod_id}", MODRINTH_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .context("Modrinth get_mod_info request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        let project: MrProject = resp
            .json()
            .await
            .context("Failed to parse Modrinth project response")?;

        Ok(project.to_mod_info())
    }

    async fn get_mod_versions(&self, mod_id: &str) -> anyhow::Result<Vec<ModVersion>> {
        let url = format!("{}/project/{mod_id}/version", MODRINTH_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .context("Modrinth get_mod_versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        let versions: Vec<MrVersion> = resp
            .json()
            .await
            .context("Failed to parse Modrinth versions response")?;

        Ok(versions
            .into_iter()
            .filter_map(|v| v.to_mod_version())
            .collect())
    }

    async fn get_version_by_id(&self, version_id: &str) -> anyhow::Result<ModVersion> {
        let url = format!("{}/version/{version_id}", MODRINTH_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .context("Modrinth get_version_by_id request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        let version: MrVersion = resp
            .json()
            .await
            .context("Failed to parse Modrinth version response")?;

        version
            .to_mod_version()
            .ok_or_else(|| anyhow::anyhow!("Modrinth version has no downloadable file"))
    }

    async fn get_multiple_versions(&self, version_ids: &[&str]) -> anyhow::Result<Vec<ModVersion>> {
        let ids = version_ids.join(",");
        let url = format!("{}/versions?ids={ids}", MODRINTH_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .context("Modrinth get_multiple_versions request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        let versions: Vec<MrVersion> = resp
            .json()
            .await
            .context("Failed to parse Modrinth versions response")?;

        Ok(versions
            .into_iter()
            .filter_map(|v| v.to_mod_version())
            .collect())
    }

    async fn get_version_changelog(&self, _mod_id: &str, version_id: &str) -> anyhow::Result<String> {
        let url = format!("{}/version/{version_id}/changelog", MODRINTH_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await
            .context("Modrinth get_version_changelog request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Modrinth API error: {status} - {body}");
        }

        Ok(resp.text().await.unwrap_or_default())
    }
}
