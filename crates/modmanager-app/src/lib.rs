use std::sync::Arc;
use tokio::sync::RwLock;

use modmanager_core::prelude::*;

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub curseforge: CurseForgeApi,
    pub modrinth: ModrinthApi,
    pub downloader: RwLock<Downloader>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let config = AppConfig::load().unwrap_or_default();
        let use_mirror = config.use_mirror;
        let downloader = Downloader::new()?.with_mirror(use_mirror);
        Ok(Self {
            config: RwLock::new(config),
            curseforge: CurseForgeApi::new()?,
            modrinth: ModrinthApi::new()?,
            downloader: RwLock::new(downloader),
        })
    }
}

#[tauri::command]
async fn search_mods(
    state: tauri::State<'_, Arc<AppState>>,
    query: Option<String>,
    game_version: Option<String>,
    loader: Option<String>,
    provider: Option<String>,
    offset: Option<u32>,
) -> Result<serde_json::Value, String> {
    let filter = SearchFilter {
        query,
        game_version: game_version.map(GameVersion::new),
        loader: loader.and_then(|l| match l.to_lowercase().as_str() {
            "forge" => Some(ModLoader::Forge),
            "neoforge" => Some(ModLoader::NeoForge),
            "fabric" => Some(ModLoader::Fabric),
            "quilt" => Some(ModLoader::Quilt),
            _ => None,
        }),
        category: None,
        offset: offset.unwrap_or(0),
        limit: 25,
    };

    let result = match provider.as_deref() {
        Some("curseforge") => state
            .curseforge
            .search(&filter)
            .await
            .map_err(|e| e.to_string())?,
        _ => state
            .modrinth
            .search(&filter)
            .await
            .map_err(|e| e.to_string())?,
    };

    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_mod_versions(
    state: tauri::State<'_, Arc<AppState>>,
    mod_id: String,
    provider: String,
) -> Result<serde_json::Value, String> {
    let versions = match provider.as_str() {
        "curseforge" => state
            .curseforge
            .get_mod_versions(&mod_id)
            .await
            .map_err(|e| e.to_string())?,
        _ => state
            .modrinth
            .get_mod_versions(&mod_id)
            .await
            .map_err(|e| e.to_string())?,
    };

    serde_json::to_value(versions).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_mod(
    state: tauri::State<'_, Arc<AppState>>,
    version: serde_json::Value,
    mod_name: String,
    provider: String,
) -> Result<String, String> {
    let version: ModVersion =
        serde_json::from_value(version).map_err(|e| e.to_string())?;

    let config = state.config.read().await;
    let mods_dir = config
        .mods_dir()
        .ok_or("No active mod directory configured".to_string())?;

    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;

    let downloader = state.downloader.read().await;
    let path = downloader
        .download_version(&version, &mods_dir)
        .await
        .map_err(|e| e.to_string())?;

    let provider_type = match provider.as_str() {
        "curseforge" => ModProvider::CurseForge,
        _ => ModProvider::Modrinth,
    };

    let metadata = ModMetadata::from_version(&version, &mod_name, provider_type);
    metadata.save(&mods_dir).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn list_local_mods(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let config = state.config.read().await;
    let mods_dir = config
        .mods_dir()
        .ok_or("No active mod directory configured".to_string())?;

    let mods = ModScanner::scan_mods_dir(&mods_dir).map_err(|e| e.to_string())?;

    serde_json::to_value(mods).map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_mod(mod_path: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&mod_path);
    let local_mod = LocalMod {
        path: path.clone(),
        filename: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        enabled: !mod_path.contains(".disabled"),
        file_size: 0,
        metadata: None,
    };

    let new_path = ModScanner::toggle_mod(&local_mod).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn delete_mod(mod_path: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(&mod_path);
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;

    let meta_path = path.with_extension("toml");
    if meta_path.exists() {
        std::fs::remove_file(meta_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn get_config(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let config = state.config.read().await;
    serde_json::to_value(&*config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_config(
    state: tauri::State<'_, Arc<AppState>>,
    config: serde_json::Value,
) -> Result<(), String> {
    let new_config: AppConfig =
        serde_json::from_value(config).map_err(|e| e.to_string())?;
    new_config.save().map_err(|e| e.to_string())?;

    let use_mirror = new_config.use_mirror;
    let mut current = state.config.write().await;
    *current = new_config;

    drop(current);

    let mut downloader = state.downloader.write().await;
    *downloader = Downloader::new().map_err(|e| e.to_string())?.with_mirror(use_mirror);

    Ok(())
}

#[tauri::command]
async fn check_updates(
    state: tauri::State<'_, Arc<AppState>>,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<serde_json::Value, String> {
    let config = state.config.read().await;
    let mods_dir = config
        .mods_dir()
        .ok_or("No active mod directory configured".to_string())?;

    let checker = UpdateChecker::new(&state.curseforge as &dyn ModPlatform, &state.modrinth as &dyn ModPlatform)
        .with_game_version(game_version.map(GameVersion::new))
        .with_loader(loader.and_then(|l| match l.to_lowercase().as_str() {
            "forge" => Some(ModLoader::Forge),
            "neoforge" => Some(ModLoader::NeoForge),
            "fabric" => Some(ModLoader::Fabric),
            "quilt" => Some(ModLoader::Quilt),
            _ => None,
        }));

    let updates = checker.check_updates(&mods_dir).await.map_err(|e| e.to_string())?;

    serde_json::to_value(updates).map_err(|e| e.to_string())
}

pub fn run() {
    let state = Arc::new(AppState::new().expect("Failed to initialize app state"));

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            search_mods,
            get_mod_versions,
            install_mod,
            list_local_mods,
            toggle_mod,
            delete_mod,
            get_config,
            update_config,
            check_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
