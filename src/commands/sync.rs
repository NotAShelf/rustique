use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use colored::{Color, Colorize};
use comfy_table::Attribute;
use rayon::prelude::*;
use serde_json::to_string_pretty;
use ureq::Agent;
use semver::{Error, Version};
use tracing::{debug, error, info, warn};
use crate::aliases::{ModFileName, ModID, ModVersion};
use crate::rustique_errors::RustiqueError;
use crate::api::api_structs::{Mod, ModInfo, Releases};
use crate::utils::{RustiqueOptions, get_current_time, extract_all_mods_metadata, elapsed_footer, notice};
use crate::api::client::{ApiClient, ModApiFetch};
use crate::config_manager::get_config;
use crate::install_manager::Install;
use crate::rustique_errors::RustiqueError::UrlParseError;
use crate::version_management::{parse_latest_version, parse_version};

#[derive(Deserialize, Serialize, Debug)]
pub struct RustiqueSyncJson {
    #[serde(rename = "RustiqueSync")]
    pub rustique_sync: HashMap<ModID, ModSyncInfo>,
    pub last_sync: String,
}

impl RustiqueSyncJson {
    pub fn new() -> RustiqueSyncJson {
        Self {
            rustique_sync: HashMap::<ModID, ModSyncInfo>::new(),
            last_sync: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModSyncInfo {
    pub file_name: ModFileName,
    pub mod_name: String,
    pub installed_version: ModVersion,
    pub latest_known_version: ModVersion,
    pub latest_download_url: String,
}


pub async fn handle_sync_call(mod_dir: &PathBuf) {
    match sync(mod_dir).await {
        Ok(_) => {}
        Err(e) => {
           error!("{}", e.to_string());
            exit(1);
        }
    }
}

pub const SYNC_FILE_NAME: &str = "rustique-sync.json";

pub fn parse_sync_file(mod_dir: &PathBuf) -> Result<RustiqueSyncJson, RustiqueError> {
    let mut file = File::open(mod_dir.join(SYNC_FILE_NAME)).map_err(|e| RustiqueError::IoError {
        context: format!("Unable to open {}", SYNC_FILE_NAME),
        source: e,
    })?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).map_err(|e| RustiqueError::IoError {
        context: format!("Failure while reading from file {}", SYNC_FILE_NAME),
        source: e
    })?;

    let json = serde_json5::from_str::<RustiqueSyncJson>(&file_contents)
        .map_err(|e| RustiqueError::JsonError {
            context: format!("Json parsing Error for {}", SYNC_FILE_NAME),
            source: e
        })?;

    Ok(json)
}




pub async fn sync(mod_dir: &PathBuf) -> Result<(), RustiqueError> {
    notice("Syncing...", Option::from(comfy_table::Color::Green), vec![Attribute::Bold]);
    let start_time = Instant::now();
    let config = get_config().read().unwrap();

    // check if rustique-sync.json exists
    // if so, parse the file for updating
    // if not, do all the sync process and then write a new file
    let file_path = mod_dir.join(SYNC_FILE_NAME);
    debug!("sync file: {}", file_path.display());

    let mut sync_data = RustiqueSyncJson {
        rustique_sync: HashMap::new(),
        last_sync: get_current_time(),
    };

    let installed_mods = extract_all_mods_metadata(mod_dir)?;

    installed_mods.iter().for_each(|(mod_filename, mod_info)| {
        let version = if let Ok(parsed_version) = parse_version(mod_info.version.clone().unwrap_or_default()) {
            parsed_version.to_string()
        } else {
            warn!("Could not parse version: {} for {}\n\rThis mod may not update correctly..", mod_info.version.clone().unwrap_or_default(), mod_filename.to_string());
            mod_info.version.clone().unwrap_or_default()
        };

        info!("VERSION Parsed: {} for {}", version, mod_info.mod_id);

        sync_data
            .rustique_sync
            .entry(mod_info.mod_id.clone())
            .or_insert_with(|| ModSyncInfo {
                installed_version: version.clone(),
                file_name: mod_filename.clone(),
                mod_name: mod_info.name.clone(),
                latest_download_url: String::new(),
                latest_known_version: String::new(),
            });
    });

    // Create API client and fetch mods in parallel using async
    let client = ApiClient::new();
    let result: HashMap<ModID, Mod> = client
        .fetch_mods_parallel(
            installed_mods.into_values().map(|m|m.mod_id).collect())
        .await?;

    result.iter().for_each(|(mod_id, mod_info): (&ModID, &Mod)| {
        let (mod_version, download_url) = parse_latest_version(&mod_info.mod_json.releases);

        sync_data
            .rustique_sync
            .entry(mod_id.clone())
            .and_modify(|sync_info| {
                sync_info.latest_known_version = mod_version.clone();
                sync_info.latest_download_url = download_url.clone();
            })
            .or_insert_with(|| ModSyncInfo {
                latest_known_version: mod_version,
                latest_download_url: download_url,
                mod_name: mod_info.mod_json.name.clone().unwrap_or_default(),
                file_name: "None".to_string(),
                installed_version: "None".to_string(),
            });
    });

    // Write the sync data to file
    let data = sync_data;
    let json = to_string_pretty(&data).map_err(|e| RustiqueError::JsonError {
        context: "Failure while making the sync json pretty".to_string(),
        source: serde_json5::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)),
    })?;

    // Use tokio's async file operations
    let mut file = tokio::fs::File::create(file_path)
        .await
        .map_err(|e| RustiqueError::IoError {
            context: format!("Error writing sync file to mod_dir: {}", mod_dir.to_string_lossy()),
            source: e,
        })?;

    tokio::io::AsyncWriteExt::write_all(&mut file, json.as_bytes())
        .await?;
        // .map_err(|e| RustiqueError::ApiError {
        //     context: format!("Error writing data to sync file: {}", file_path.to_string_lossy()),
        //     source: e,
        // })?;

    if config.show_execution_time {
        elapsed_footer(start_time, "Sync");
    }

    Ok(())
}
