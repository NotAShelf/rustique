use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use futures::future::join_all;
use lithic_core::api::client::ApiClient;
use lithic_core::api::structs::{ModApi, ModInfo, ModsSearchFile};
use lithic_core::config::manager::{Config, get_config};
use lithic_core::consts::{FILE_GAME_VERSION_SYNC, FILE_LITHIC_SYNC, FILE_MOD_SEARCH_SYNC};
use lithic_core::installer::manager::{Install, install_manager};
use lithic_core::search::SearchQuery;
use lithic_core::sync::structs::{GameVersionSync, LithicSyncJson, ModSyncInfo};
use lithic_core::utils::{
    extract_all_mods_metadata, get_current_time, parse_json_file, prettify, write_json_file,
};
use lithic_core::version::filter::{VersionFilter, minor_versions_at_least, unique_minor_versions};
use lithic_core::version::manager::parse_latest_version;

const FAVORITES_FILE: &str = "lithic-gui-favorites.json";

fn err(e: impl ToString) -> String {
    e.to_string()
}

#[derive(Debug, Clone, Default)]
pub struct SettingsData {
    pub mod_dir: String,
    pub game_download_dir: String,
    pub pinned_game_version: String,
    pub zip_mod_files: bool,
    pub backup_mods: bool,
    pub backup_mods_dir: String,
    pub notify_of_unzipped_mods: bool,
    pub check_for_updates: bool,
    pub show_execution_time: bool,
    pub modpack_dir: String,
}

pub async fn load_installed() -> Result<HashMap<String, ModSyncInfo>, String> {
    let mod_dir = {
        let config = get_config().read().await;
        PathBuf::from(&config.mod_dir)
    };
    load_installed_from(mod_dir).await
}

pub async fn load_installed_from(mod_dir: PathBuf) -> Result<HashMap<String, ModSyncInfo>, String> {
    if mod_dir.as_os_str().is_empty() || !mod_dir.exists() {
        return Ok(HashMap::new());
    }
    let sync_file = mod_dir.join(FILE_LITHIC_SYNC);
    if sync_file.exists() {
        let data = parse_json_file::<LithicSyncJson>(&sync_file)
            .await
            .map_err(err)?;
        return Ok(data.lithic_sync);
    }
    build_basic_installed(&mod_dir).await
}

async fn build_basic_installed(mod_dir: &Path) -> Result<HashMap<String, ModSyncInfo>, String> {
    let installed = extract_all_mods_metadata(mod_dir, true)
        .await
        .map_err(err)?;
    let map = installed
        .into_iter()
        .map(|(file_name, info)| {
            let key = if info.mod_id.is_empty() {
                file_name.to_string()
            } else {
                info.mod_id.to_string()
            };
            let sync = ModSyncInfo {
                file_name: file_name.clone(),
                mod_name: info.name,
                installed_version: info.version.unwrap_or_default(),
                ..ModSyncInfo::default()
            };
            (key, sync)
        })
        .collect();
    Ok(map)
}

pub async fn sync_mods(mod_dir: PathBuf) -> Result<HashMap<String, ModSyncInfo>, String> {
    if mod_dir.as_os_str().is_empty() || !mod_dir.exists() {
        return Err(
            "Mods directory is not set or does not exist. Configure it in Settings.".to_string(),
        );
    }
    let scanned = build_basic_installed(&mod_dir).await?;

    let sync_file = mod_dir.join(FILE_LITHIC_SYNC);
    if !sync_file.exists() {
        return Ok(scanned);
    }

    let mut sync_map = parse_json_file::<LithicSyncJson>(&sync_file)
        .await
        .map(|d| d.lithic_sync)
        .unwrap_or_default();

    for (id, info) in scanned {
        sync_map.entry(id).or_insert(info);
    }

    let sync = LithicSyncJson {
        lithic_sync: sync_map.clone(),
        last_sync: get_current_time(),
    };
    sync.save(&sync_file).await.map_err(err)?;

    Ok(sync_map)
}

pub async fn update_all(mod_dir: PathBuf) -> Result<(), String> {
    if mod_dir.as_os_str().is_empty() || !mod_dir.exists() {
        return Err("Mods directory not configured.".to_string());
    }
    let sync_file = mod_dir.join(FILE_LITHIC_SYNC);
    if !sync_file.exists() {
        return Err("No sync data found. Run sync first.".to_string());
    }
    let sync_data = parse_json_file::<LithicSyncJson>(&sync_file)
        .await
        .map_err(err)?;

    let mods_needing_update: Vec<Install> = sync_data
        .lithic_sync
        .iter()
        .filter(|(_, info)| {
            !info.is_symlink
                && !info.latest_known_version.is_empty()
                && info.installed_version != info.latest_known_version
        })
        .map(|(mod_id, info)| Install {
            mod_id: mod_id.clone().into(),
            mod_name: info.mod_name.clone().into(),
            download_url: info.latest_download_url.clone().into(),
            version_to_install: info.latest_known_version.clone(),
            current_file_path: Some(mod_dir.join(&info.file_name)),
        })
        .collect();

    if mods_needing_update.is_empty() {
        return Ok(());
    }

    install_manager(mod_dir, mods_needing_update, sync_data.lithic_sync)
        .await
        .map(|_| ())
        .map_err(err)
}

pub async fn delete_mod(mod_dir: PathBuf, file_name: String) -> Result<String, String> {
    let file_path = mod_dir.join(&file_name);
    if file_path.exists() {
        tokio::fs::remove_file(&file_path).await.map_err(err)?;
    }

    let sync_file = mod_dir.join(FILE_LITHIC_SYNC);
    if sync_file.exists() {
        if let Ok(mut data) = parse_json_file::<LithicSyncJson>(&sync_file).await {
            data.lithic_sync
                .retain(|_, info| info.file_name != file_name);
            let _ = data.save(&sync_file).await;
        }
    }

    Ok(file_name)
}

pub async fn refresh_browse() -> Result<Vec<ModApi>, String> {
    let path = Config::get_path().join(FILE_MOD_SEARCH_SYNC);
    fetch_and_cache_mods(&path).await
}

pub async fn load_browse() -> Result<Vec<ModApi>, String> {
    let search_file_path = Config::get_path().join(FILE_MOD_SEARCH_SYNC);

    if search_file_path.exists() {
        if let Ok(data) = parse_json_file::<ModsSearchFile>(&search_file_path).await {
            if !data.mods.is_empty() {
                return Ok(data.mods);
            }
        }
    }
    fetch_and_cache_mods(&search_file_path).await
}

async fn fetch_and_cache_mods(path: &Path) -> Result<Vec<ModApi>, String> {
    let client = ApiClient::new();
    let all = client.fetch_all_mods().await.map_err(err)?;
    let file_data = ModsSearchFile {
        mods: all.mods.clone(),
        last_sync: get_current_time(),
    };
    let json = prettify(&file_data, "Mods Search DB").map_err(err)?;
    let config_dir = Config::get_path();
    write_json_file(path, json, &config_dir)
        .await
        .map_err(err)?;
    Ok(all.mods)
}

pub fn search_mods(all_mods: &[ModApi], query: &str) -> Vec<ModApi> {
    if query.trim().is_empty() {
        return all_mods.to_vec();
    }
    SearchQuery::new()
        .add_text_search(query.to_string())
        .execute(all_mods)
}

pub async fn install_mod(mod_dir: PathBuf, mod_id: String) -> Result<String, String> {
    if mod_dir.as_os_str().is_empty() || !mod_dir.exists() {
        return Err("Mods directory not configured. Set it in Settings first.".to_string());
    }
    let sync_file = mod_dir.join(FILE_LITHIC_SYNC);
    let mut sync_map: HashMap<String, ModSyncInfo> = if sync_file.exists() {
        parse_json_file::<LithicSyncJson>(&sync_file)
            .await
            .map(|d| d.lithic_sync)
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    let client = ApiClient::new();
    let mod_info = client.fetch_mod(&mod_id).await.map_err(err)?;
    let mod_name = mod_info
        .mod_json
        .name
        .clone()
        .unwrap_or_else(|| mod_id.clone());

    let (version, download_url, _, _) = parse_latest_version(&mod_info.mod_json.releases);

    let to_install = vec![Install {
        mod_id: mod_id.clone().into(),
        mod_name: mod_name.clone().into(),
        download_url: download_url.clone(),
        version_to_install: version.clone(),
        current_file_path: None,
    }];

    let installed_list = install_manager(mod_dir.clone(), to_install, sync_map.clone())
        .await
        .map_err(err)?;

    for inst in &installed_list {
        let Some(path) = &inst.installed_file_path else {
            continue;
        };
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if file_name.is_empty() {
            continue;
        }
        let is_primary = inst.mod_id.as_ref() == mod_id;
        sync_map.insert(
            inst.mod_id.to_string(),
            ModSyncInfo {
                file_name: file_name.into(),
                mod_name: inst.mod_name.to_string(),
                installed_version: if is_primary && inst.install_version.is_empty() {
                    version.clone()
                } else {
                    inst.install_version.clone()
                },
                latest_known_version: if is_primary {
                    version.clone()
                } else {
                    inst.install_version.clone()
                },
                latest_download_url: if is_primary {
                    download_url.to_string()
                } else {
                    String::new()
                },
                ..ModSyncInfo::default()
            },
        );
    }

    let sync = LithicSyncJson {
        lithic_sync: sync_map,
        last_sync: get_current_time(),
    };
    sync.save(&sync_file).await.map_err(err)?;

    Ok(mod_name)
}

pub async fn load_favorites() -> Result<HashSet<String>, String> {
    let path = Config::get_path().join(FAVORITES_FILE);
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let data = tokio::fs::read_to_string(&path).await.map_err(err)?;
    serde_json::from_str(&data).map_err(err)
}

pub async fn save_favorites(favorites: HashSet<String>) -> Result<(), String> {
    let path = Config::get_path().join(FAVORITES_FILE);
    let data = serde_json::to_string(&favorites).map_err(err)?;
    tokio::fs::write(&path, data).await.map_err(err)
}

pub async fn export_favorites(favorites: HashSet<String>) -> Result<String, String> {
    let path = Config::get_path().join("lithic-favorites-export.txt");
    let mut ids: Vec<&String> = favorites.iter().collect();
    ids.sort();
    let content = ids
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(&path, content).await.map_err(err)?;
    Ok(path.display().to_string())
}

pub async fn load_packs() -> Result<(Vec<String>, Vec<String>), String> {
    let config = get_config().read().await;
    let disabled = config.modpacks.disabled.clone();
    let enabled = config.modpacks.enabled.clone();
    Ok((disabled, enabled))
}

pub async fn enable_pack(id: String) -> Result<String, String> {
    let mut config = get_config().write().await;
    if !config.modpacks.enabled.contains(&id) {
        config.modpacks.enabled.push(id.clone());
        config.modpacks.disabled.retain(|m| m != &id);
        config.save(None).map_err(err)?;
    }
    Ok(format!("{id} enabled"))
}

pub async fn disable_pack(id: String) -> Result<String, String> {
    let mut config = get_config().write().await;
    config.modpacks.enabled.retain(|m| m != &id);
    if !config.modpacks.disabled.contains(&id) {
        config.modpacks.disabled.push(id.clone());
    }
    config.save(None).map_err(err)?;
    Ok(format!("{id} disabled"))
}

pub async fn create_pack(
    mod_dir: PathBuf,
    name: String,
    pack_id: String,
    version: String,
) -> Result<String, String> {
    let modpack_dir = {
        let config = get_config().read().await;
        config.modpacks.modpack_dir.clone()
    };

    if modpack_dir.is_empty() {
        return Err("Modpack directory not configured. Set it in Settings first.".to_string());
    }

    let installed = load_installed_from(mod_dir).await?;
    let dependencies: HashMap<String, String> = installed
        .into_iter()
        .filter(|(id, _)| !id.is_empty())
        .map(|(id, info)| (id, info.installed_version.to_string()))
        .collect();

    let save_path = std::path::Path::new(&modpack_dir).join("mypacks");
    tokio::fs::create_dir_all(&save_path).await.map_err(err)?;

    let pack_info = ModInfo {
        name: name.clone(),
        mod_id: pack_id.clone().into(),
        version: if version.is_empty() {
            None
        } else {
            Some(version.into())
        },
        dependencies: dependencies
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        ..ModInfo::default()
    };

    pack_info
        .build_modpack(save_path.clone(), pack_id.clone().into())
        .await
        .map_err(err)?;

    {
        let mut config = get_config().write().await;
        if !config.modpacks.disabled.contains(&pack_id) {
            config.modpacks.disabled.push(pack_id.clone());
        }
        config.save(None).map_err(err)?;
    }

    Ok(format!(
        "Created modpack '{name}' at {}",
        save_path.display()
    ))
}

pub async fn load_game_versions() -> Result<Vec<String>, String> {
    let path = Config::get_path().join(FILE_GAME_VERSION_SYNC);

    let raw: Vec<String> = if path.exists() {
        parse_json_file::<GameVersionSync>(&path)
            .await
            .map(|gvs| gvs.game_versions)
            .unwrap_or_default()
    } else {
        let client = ApiClient::new();
        let set = client.fetch_game_versions().await.map_err(err)?;
        let mut sorted: Vec<String> = set.into_iter().collect();
        sorted.sort();
        let gvs = GameVersionSync {
            game_versions: sorted.clone(),
            last_sync: get_current_time(),
        };
        let json = prettify(&gvs, "game versions").map_err(err)?;
        let _ = write_json_file(&path, json, &Config::get_path()).await;
        sorted
    };

    Ok(unique_minor_versions(&raw))
}

pub async fn fetch_versioned_browse(
    filter: VersionFilter,
    all_minor_versions: Vec<String>,
) -> Result<Vec<ModApi>, String> {
    let client = ApiClient::new();

    match &filter {
        VersionFilter::Any => {
            let all = client.fetch_all_mods().await.map_err(err)?;
            Ok(all.mods)
        }
        VersionFilter::Exact(v) => {
            let result = client.fetch_mods_with_gameversion(v).await.map_err(err)?;
            Ok(result.mods)
        }
        VersionFilter::AtLeast(min_v) => {
            let targets: Vec<String> = minor_versions_at_least(&all_minor_versions, min_v)
                .into_iter()
                .cloned()
                .collect();

            if targets.is_empty() {
                return Ok(Vec::new());
            }

            let fetches = targets.into_iter().map(|v| {
                let client = client.clone();
                async move { client.fetch_mods_with_gameversion(&v).await }
            });

            let results = join_all(fetches).await;

            let mut seen = HashSet::new();
            let mut combined = Vec::new();
            for result in results {
                if let Ok(mods_result) = result {
                    for m in mods_result.mods {
                        if seen.insert(m.mod_id) {
                            combined.push(m);
                        }
                    }
                }
            }
            Ok(combined)
        }
    }
}

pub async fn load_settings() -> Result<SettingsData, String> {
    let config = get_config().read().await;
    Ok(SettingsData {
        mod_dir: config.mod_dir.clone(),
        game_download_dir: config.game_download_dir.clone(),
        pinned_game_version: config.pinned_game_version.clone(),
        zip_mod_files: config.zip_mod_files,
        backup_mods: config.backup_mods,
        backup_mods_dir: config.backup_mods_dir.clone(),
        notify_of_unzipped_mods: config.notify_of_unzipped_mods,
        check_for_updates: config.check_for_updates,
        show_execution_time: config.show_execution_time,
        modpack_dir: config.modpacks.modpack_dir.clone(),
    })
}

pub async fn save_settings(s: SettingsData) -> Result<(), String> {
    let mut config = get_config().write().await;
    config.mod_dir = s.mod_dir;
    config.game_download_dir = s.game_download_dir;
    config.pinned_game_version = s.pinned_game_version;
    config.zip_mod_files = s.zip_mod_files;
    config.backup_mods = s.backup_mods;
    config.backup_mods_dir = s.backup_mods_dir;
    config.notify_of_unzipped_mods = s.notify_of_unzipped_mods;
    config.check_for_updates = s.check_for_updates;
    config.show_execution_time = s.show_execution_time;
    config.modpacks.modpack_dir = s.modpack_dir;
    config.save(None).map_err(err)
}
