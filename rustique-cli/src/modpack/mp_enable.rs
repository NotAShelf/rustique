
use std::path::Path;
use comfy_table::{Attribute, Color};
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;

#[cfg(windows)]
use is_elevated::is_elevated;

#[cfg(windows)]
use std::process::exit;
use rustique_core::aliases::{FileName, ModID};
use rustique_core::config::config_manager::get_config;
use rustique_core::information_utils::{display_table, notice, CellData};
use rustique_core::rustique_errors::RustiqueError;
use rustique_core::symlink_manager::SymlinkManager;
use rustique_core::traits::ref_ext::PathRef;
use rustique_core::utils::extract_all_mods_metadata;

pub async fn mp_enable(mpk_id: ModID, mod_dir: impl PathRef, force: bool) -> Result<String, RustiqueError> {
    
   
    #[cfg(windows)]
    if !is_elevated() {
        notice("In order to enable modpacks, Rustique uses symlinks which require admin permissions on Windows. Please run Rustique with admins right and try again.", Some(Color::Red), vec![Attribute::Bold]);
        exit(1);
    }
    
    let config = get_config().read().await;
    let mod_pack_install_dir = Path::new(&config.modpacks.modpack_dir).join("installed");
    let full_dir_with_mpk_id = mod_pack_install_dir.join(&mpk_id);
    
    if !full_dir_with_mpk_id.exists() {
        return Err(RustiqueError::SimpleError(format!("Modpack {} doesn't exist in {}. Run 'Rustique modpack list' or 'Rustique modpack local list' to view your installed modpacks.", &mpk_id, &mod_pack_install_dir.display())));
    }
    
    // check if a modpack already exists
    // if so, notify the user and tell them to either disable the current one OR use modpack enable -f to force the use and warn about using multiple
    
    // Is it already enabled?
    if config.modpacks.enabled.contains(&mpk_id) {
        notice(format!("Modpack: [{}] is already enabled. Did you mean to disable it?", &mpk_id), Some(Color::Yellow), vec![Attribute::Bold]);
        return Err(RustiqueError::SimpleError("Modpack already enabled".into()));
    }
    
    // Is it even installed??
    if !config.modpacks.disabled.contains(&mpk_id) {
        notice(format!("Modpack: [{}] is not installed! Use [Rustique modpack install {}] to install it first.", &mpk_id, &mpk_id), Some(Color::Yellow), vec![Attribute::Bold]);
        return Err(RustiqueError::SimpleError("Modpack needs to be installed first".into()))
    }
    
    // Is anything else enabled?
    if !config.modpacks.enabled.is_empty() && !force {
        
        display_table(vec![
            (CellData::new("You already have the following modpack(s) enabled: ".into(), Some(Color::Yellow), vec![], None),
            CellData::new(config.modpacks.enabled.join(","), Some(Color::Magenta), vec![], None))
        ], Some(UTF8_HORIZONTAL_ONLY));
        
        notice("Run this command again with --force to enable it anyway..", Some(Color::Yellow), vec![]);
        return Err(RustiqueError::SimpleError(format!("Modpacks already enabled {}", config.modpacks.enabled.join(","))));
        
    }
    
    // we know that the modpack is installed and IS NOT enabled
    // lets enable it
    
    // get list of mods for the modpack
    // create symlinks is the Mods folder
    // return the modpack id that was enabled
    
    let mod_list: Vec<FileName> = extract_all_mods_metadata(&full_dir_with_mpk_id, false).await?
        .keys()
        .cloned()
        .collect();
    
    for m in mod_list {
        let target = &full_dir_with_mpk_id.join(&m);
        let link = mod_dir.as_ref().join(&m);
        SymlinkManager::create(target, link).await?;
    }
    
    
    Ok(mpk_id)
}