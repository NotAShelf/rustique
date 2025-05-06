use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::api::client::ApiClient;
use crate::commands::sync::{parse_sync_file, sync, ModSyncInfo};
use crate::utils::{delete_file, RustiqueOptions, elapsed_footer, notice};
use rayon::prelude::*;
use std::process::exit;
use std::time::Instant;
use colored::Colorize;
use comfy_table::{Attribute, Color};
use tracing::{error, info, warn};
use url::{form_urlencoded, Url};
use crate::aliases::ModID;
use crate::api::download::download_requested_mods;
use crate::commands::install::{install_mod, install_mods, InstallOrUpdate};
use crate::config_manager::get_config;
use crate::install_manager::{install_manager, Install};
use crate::rustique_errors::RustiqueError;



pub async fn update_mods(mod_dir: &PathBuf, update_mod_ids: Vec<ModID>, _keep_old_files: bool) -> Result<(), RustiqueError> {
    let start_time = Instant::now();
    let config = get_config().read().unwrap();
    let sync_data = parse_sync_file(mod_dir);

    if sync_data.is_ok() {
        notice("Updating mods...", Option::from(Color::Green), vec![Attribute::Bold]);
        let sync_data = sync_data?;
        let mut mods_to_check_update: HashMap<ModID, ModSyncInfo> = HashMap::new();
        let mut updates_exist = false;

        if !update_mod_ids.is_empty() {
            update_mod_ids.iter().for_each(|typed_mod_id| {
                let mod_sync_data = &sync_data.rustique_sync;
                // user typed in a valid typed_mod_id so violet is happy now
                let typed_mod_id = typed_mod_id.to_lowercase();
                if mod_sync_data.contains_key(&typed_mod_id) {
                    mods_to_check_update.entry(typed_mod_id.clone()).or_insert(mod_sync_data[&typed_mod_id].clone());
                    updates_exist = true;
                } else {
                    println!("{} is not a valid mod_id!", &typed_mod_id.red());
                }
            });
        } else {
            mods_to_check_update = sync_data.rustique_sync.clone();
            updates_exist = true;
        }

        if !updates_exist {
            return Err(RustiqueError::SimpleError(String::from("No valid update ids or the mod dir is empty..\n\r")))
        }

        let all_installed_mods: HashMap<ModID, ModSyncInfo> = mods_to_check_update.clone();
        info!("all_installed_mods: {:#?}", all_installed_mods);

        let final_mod_update_list: Vec<Install> = mods_to_check_update
            .into_iter()
            .filter_map(|(mod_id, mod_sync_info)| {

            if mod_sync_info.latest_known_version != mod_sync_info.installed_version && !mod_id.is_empty() {
                Some(Install {
                    mod_id,
                    mod_name: mod_sync_info.mod_name.clone(),
                    version_to_install: mod_sync_info.latest_known_version.clone(),
                    download_url: mod_sync_info.latest_download_url.clone()
                })
            } else {
                None
            }
        }).collect();

        info!("final_mod_update_list: {:#?}", final_mod_update_list);


        install_manager(mod_dir, final_mod_update_list.clone(), all_installed_mods).await?;



        // if !_keep_old_files {
        //     // Using tokio::fs for file deletion
        //     for i in final_mod_update_list.iter() {
        //         if !a.is_empty() {
        //             let file_path = &mod_dir.clone().join(mod_sync_info.file_name.to_string());
        //             match tokio::fs::remove_file(file_path).await {
        //                 Ok(_) => {
        //                     info!("{} {}", &mod_sync_info.file_name.bright_yellow(), "deleted successfully!".green() );
        //                 },
        //                 Err(e) => {
        //                     error!("{} {}: {}", "Error deleting file".red(), file_path.display().to_string().bright_yellow(), e);
        //                 }
        //             }
        //         } else {
        //             warn!("Mod {} is missing a mod id in the modinfo.json file. Rustique will be unable to update or manage this mod. ", &mod_sync_info.file_name.to_string().red().bold());
        //         }
        //     }
        // }
    } else {
        println!("{} {} {}", "Looks like you need to run".bright_yellow(), "'Rustique sync'".bright_blue().bold(), "first".yellow());
        exit(1);
    }

    if config.show_execution_time {
        elapsed_footer(start_time, "Update");
    }

    Ok(())
}

