

// Delete a modpack. if the pack is enabled, return an error stating they need to disable it first,
// this prevents people from unintentionally deleting an active modpack.

use std::collections::HashMap;
use std::path::Path;
use comfy_table::Color;
use crate::aliases::{ModFileName, ModID};
use crate::api::api_structs::ModInfo;
use crate::config::config_manager::get_config;
use crate::information_utils::notice;
use crate::rustique_errors::RustiqueError;
use crate::traits::ref_ext::PathRef;
use crate::utils::{delete_file, extract_all_mods_metadata};

pub async fn delete_mpk_cmd(mpk_id: ModID) -> Result<(), RustiqueError> {
    // verify the modpack is installed
    // verify its not enable
    // if yes, delete the .zip from mypacks
    // check if mods dir exist and delete it from installed
    // update config to 
    
    let mut config = get_config().write().await;
    
    if config.modpacks.enabled.contains(&mpk_id) {
        notice(format!("{mpk_id} is currently enabled! Disable it first before attempting to delete it."), Some(Color::Yellow), vec![]);
    }
    
    if !config.modpacks.disabled.contains(&mpk_id) {
        notice(format!("{mpk_id} is not installed. Did you misspell it?"), Some(Color::Yellow), vec![]);
    }
   
    let p = config.modpacks.modpack_dir.clone();
    let base_dir = Path::new(&p);
    
    if !base_dir.exists() {
        notice("Your modpacks directory does not exist! 'Run Rustique config list' to see what its set to.".to_string(), Some(Color::Red), vec![]);
    }
    
    let mpk_mods_dir = base_dir.join("installed").join(&mpk_id);
    if mpk_mods_dir.exists() {
        tokio::fs::remove_dir_all(&mpk_mods_dir).await?;
    }
    
    let packs = extract_all_mods_metadata(&base_dir.join("packs"), false).await?;
   
    if let Ok(m) = check_and_remove(&mpk_id, packs, &mpk_mods_dir).await {
        config.modpacks.disabled.retain(|m| m != &mpk_id);
    }
    
    
   let my_packs = extract_all_mods_metadata(&base_dir.join("mypacks"), false).await?;
   
    if let Ok(m) = check_and_remove(&mpk_id, my_packs, &mpk_mods_dir).await {
        config.modpacks.disabled.retain(|m| m != &mpk_id);
    }
    
    config.save(None)?;
    drop(config);
    
    
    Ok(())
}

async fn check_and_remove(mpk_id: &ModID, mpk_data: HashMap<ModFileName, ModInfo>, mpk_mods_dir: impl PathRef) -> Result<&ModID, RustiqueError> {
    for (filename, mod_info) in mpk_data {
        if &mod_info.mod_id == mpk_id {
            delete_file(&mpk_mods_dir.as_ref().join(&filename)).await?;
            return Ok(mpk_id);
        }
    }
    Err(RustiqueError::SimpleError("Modpack not found".to_string()))
}