use crate::aliases::{ModFileName, ModID, ModVersion};
use crate::rustique_errors::RustiqueError;
use crate::utils::{extract_all_mods_metadata, find_missing_dependencies, extract_zip_metadata, notice, elapsed_footer};
use colored::Colorize;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use comfy_table::{Attribute, Color};
use tracing::{debug, error, info, warn};
use rayon::prelude::*;
use crate::api::api_structs::ModInfo;
use crate::api::client::ApiClient;
use crate::commands::sync::{get_sync_data, parse_json_file, ModSyncInfo, RustiqueSyncJson};
use crate::config_manager::get_config;
use crate::install_manager::{install_manager, Install};
use crate::rustique_errors::RustiqueError::SimpleError;
use crate::version_management::parse_latest_version;


// Report if trying install a mod that already exists
// Use -f to force an installation
// add way to set the version you want to download
pub async fn install_cmd(mod_dir: &PathBuf, mut mods_requested: Vec<ModID>, force: bool) -> Result<(), RustiqueError> {

    // get sync data
    let sync_data = get_sync_data(mod_dir).await?;

    let installed_mods = sync_data.rustique_sync.clone();
    // remove any mods from mods_requested if the exist in installed_mods

    let mods_requested_cleaned : Vec<ModID>  = mods_requested.iter().filter(|&id| !installed_mods.contains_key(id)).cloned().collect();

    if mods_requested.is_empty() {
        notice("Looks like you have all the mods requested. If you would like to reinstall them, run this command again with --force", Some(comfy_table::Color::Yellow), vec![]);
        return Err(SimpleError("No mods to install".to_string()))
    }

    let client = ApiClient::new();

    // get the download urls for all requested mods
    let result = client.fetch_mods_parallel(mods_requested_cleaned).await?;

    let mut mods_requested: Vec<Install> =
        result.into_iter().map(|(mod_id, mod_info)| {
            let (version, download_url) = parse_latest_version(&mod_info.mod_json.releases);
            Install {
                mod_id: mod_id.clone(),
                mod_name: mod_info.mod_json.name.unwrap_or_default(),
                version_to_install: version,
                download_url: download_url.clone(),
                current_file_path: None,
            }
        }).collect();


    info!("Mods requested {:?}", mods_requested);

    install_manager(&mod_dir, mods_requested.clone(), installed_mods).await?;

    Ok(())
}


pub async fn install_missing_deps(mod_dir: &PathBuf, mods_requested: Vec<ModID>) -> Result<(), RustiqueError> {

    // get all installed mod info
    // retrieve all dependencies
    // send missing ones to install_manager()

    let installed_mods = extract_all_mods_metadata(mod_dir)?;
    let sync_data = get_sync_data(mod_dir).await?.rustique_sync.clone();
    let id_vec: Vec<ModID> = sync_data.keys().cloned().collect();


    // if there are reports of slowness is this section .values().par_bridge()...flat_map_iter() could be used to speed it up
    // this is prob not an issue even with a lot of mods as the data is all in memory at this point
    let missing_deps: Vec<Install> = installed_mods
        .values()
        .filter(|mod_info| mods_requested.is_empty() || mods_requested.contains(&mod_info.mod_id))
        .flat_map(|mod_info| {
            mod_info.dependencies.as_ref()
                .map(|hm| hm.iter()
                    .filter_map(|(mod_id, version)|
                        if !mod_id.contains("game")
                            && !mod_id.contains("survival")
                            && !mod_id.contains("creative")
                            && !id_vec.contains(&mod_id) {
                            Some(Install {
                                mod_id: mod_id.clone(),
                                mod_name: "".to_string(),
                                version_to_install: version.clone(),
                                download_url: "".to_string(),
                                current_file_path: None,
                            })
                        } else {
                            None
                        }).collect::<Vec<_>>()
                ).unwrap_or_default()
                .into_iter()
    }).collect();


    info!("deps: {:?}", missing_deps);

    install_manager(&mod_dir, missing_deps, sync_data).await?;

    Ok(())
}
