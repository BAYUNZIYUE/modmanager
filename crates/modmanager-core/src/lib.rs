pub mod api;
pub mod config;
pub mod download;
pub mod local;
pub mod models;
pub mod update;

pub mod prelude {
    pub use crate::api::{BmclapiMirror, CurseForgeApi, ModPlatform, ModrinthApi};
    pub use crate::config::AppConfig;
    pub use crate::download::Downloader;
    pub use crate::local::scanner::ModScanner;
    pub use crate::local::{LocalMod, ModMetadata};
    pub use crate::models::*;
    pub use crate::update::{UpdateChecker, UpdateInfo};
}
