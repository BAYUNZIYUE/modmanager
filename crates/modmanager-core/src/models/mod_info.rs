use serde::{Deserialize, Serialize};

use super::{GameVersion, ModLoader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModProvider {
    CurseForge,
    Modrinth,
    Bmclapi,
}

impl ModProvider {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::CurseForge => "CurseForge",
            Self::Modrinth => "Modrinth",
            Self::Bmclapi => "BMCLAPI",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub provider: ModProvider,
    pub authors: Vec<ModAuthor>,
    pub icon_url: Option<url::Url>,
    pub website_url: Option<url::Url>,
    pub source_url: Option<url::Url>,
    pub issues_url: Option<url::Url>,
    pub downloads: u64,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModAuthor {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub query: Option<String>,
    pub game_version: Option<GameVersion>,
    pub loader: Option<ModLoader>,
    pub category: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            query: None,
            game_version: None,
            loader: None,
            category: None,
            offset: 0,
            limit: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub mods: Vec<ModInfo>,
    pub total_count: u32,
    pub offset: u32,
    pub limit: u32,
}
