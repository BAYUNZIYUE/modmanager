pub mod bmclapi;
pub mod curseforge;
pub mod modrinth;

pub use bmclapi::BmclapiMirror;
pub use curseforge::CurseForgeApi;
pub use modrinth::ModrinthApi;

use crate::models::{
    ModInfo, ModProvider, ModVersion, SearchFilter, SearchResult,
};

#[async_trait::async_trait]
pub trait ModPlatform: Send + Sync {
    fn provider(&self) -> ModProvider;

    async fn search(&self, filter: &SearchFilter) -> anyhow::Result<SearchResult>;

    async fn get_mod_info(&self, mod_id: &str) -> anyhow::Result<ModInfo>;

    async fn get_mod_versions(&self, mod_id: &str) -> anyhow::Result<Vec<ModVersion>>;

    async fn get_version_by_id(&self, version_id: &str) -> anyhow::Result<ModVersion>;

    async fn get_multiple_versions(&self, version_ids: &[&str]) -> anyhow::Result<Vec<ModVersion>>;

    async fn get_version_changelog(&self, mod_id: &str, version_id: &str) -> anyhow::Result<String>;
}
