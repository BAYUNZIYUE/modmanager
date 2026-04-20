use serde::{Deserialize, Serialize};

use super::{GameVersion, ModLoader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

impl VersionType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "release" | "stable" => Self::Release,
            "beta" => Self::Beta,
            "alpha" => Self::Alpha,
            _ => Self::Release,
        }
    }

    pub fn curseforge_id(self) -> i32 {
        match self {
            Self::Release => 1,
            Self::Beta => 2,
            Self::Alpha => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Client,
    Server,
    Universal,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub mod_id: String,
    pub version_number: String,
    pub version_name: String,
    pub version_type: VersionType,
    pub game_versions: Vec<GameVersion>,
    pub loaders: Vec<ModLoader>,
    pub download_url: url::Url,
    pub filename: String,
    pub file_size: u64,
    pub hash_sha1: Option<String>,
    pub hash_sha512: Option<String>,
    pub hash_md5: Option<String>,
    pub date: chrono::DateTime<chrono::Utc>,
    pub changelog: Option<String>,
    pub side: Side,
    pub dependencies: Vec<ModDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub dependency_type: DependencyType,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}
