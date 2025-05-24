
// mp_install calls normal install, except it gathers a list of all the required mods before hand 
// and sets the install location to be the modpack specific folder

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;
use comfy_table::{Attribute, Color};
use owo_colors::OwoColorize;
use tracing::{debug, error, info, warn};
use crate::api::client::ApiClient;
use crate::api::download::download_requested_mods;
use crate::commands::arg_structs::modpack_args::{MPCreateArgs, MPInstallArgs};
use crate::commands::install::install_cmd;
use crate::commands::search::{parse_search_file, SearchQuery};
use crate::commands::sync::{handle_sync_call, sync, ModSyncInfo, RustiqueSyncJson, SYNC_FILE_NAME};
use crate::config::config_manager::get_config;
use crate::information_utils::{command_output, display_table, elapsed_footer, notice};
use crate::install_manager::{install_manager, Install, Installed};
use crate::modpack::modpack_toml::ModPackToml;
use crate::rustique_errors::RustiqueError;
use crate::utils::{extract_all_mods_metadata, extract_zip_metadata, parse_json_file};
use crate::version_management::{parse_download_url_from_version, parse_latest_version};

pub async fn mp_install(args: MPInstallArgs) -> Result<(), RustiqueError> {
    let start_time = Instant::now();
    // installing the modpack with this function will do the following:
    // Save the modpack.zip (the modpack from the mods website) to modpacks/packs
    // Once the modpack is installed, it will download all the mods associated with the modpack  
    // to the location [modpacks/installed/modpack_id/*] 
    let config = get_config().read().await;
   
    let client = ApiClient::new();
    
    let mod_info = client.fetch_mod(&args.mod_id).await?;

    let installed_dir = Path::new(&config.modpacks_dir).join("installed");

    
    let (version, download_url, _) = parse_latest_version(&mod_info.mod_json.releases);
    
    let install_modpack = Install {
        mod_id: mod_info.mod_json.mod_id.clone().to_string(),
        mod_name: mod_info.mod_json.name.clone().unwrap_or_default(),
        version_to_install: version,
        download_url,
        current_file_path: None,
    };
    
    // download the modpack first, then install the dependencies

    let packs_dir = Path::new(&config.modpacks_dir).join("packs");
    let Some(modpack) = download_requested_mods(&packs_dir, &mut vec![install_modpack], &client).await?.into_iter().next() else {
            return Err(RustiqueError::SimpleError("Modpack download failure..".into()));
        };

    if let Some(modpack_packs_path) = modpack.installed_file_path {
        let modpack_info = extract_zip_metadata::<ModPackToml>(&modpack_packs_path, "modpack.toml").inspect_err(|e| {
            // bad or malformed modpack? maybe someone created it manually, 
            // TODO should rustique use its own modpack.toml file? 
        })?;

        // The modpack is installed to the correct place, install all dependencies

        let modpack_mod_path = installed_dir.join(&modpack_info.modpack.mpk_id);

        if !modpack_mod_path.exists() {
            info!("Created {modpack_mod_path:?}");
            fs::create_dir_all(&modpack_mod_path)?;
        }

        // grab the mod ids from the modpack
        let mods = modpack_info.mods.values().map(|m| m.mod_id.clone()).collect();
        info!("MODS: {mods:?}");
        let deps = client.fetch_mods_parallel(mods).await?;

        let install_mp_mods: Vec<Install> = deps.iter().filter_map(|(mod_id, mod_api)| {
            if let Some(mp_mod) = modpack_info.mods.values().find(|u|u.mod_id.eq(mod_id)) {
                let download_url = match parse_download_url_from_version(&mod_api.mod_json.releases, &mp_mod.version) {
                    Ok(download_url) => download_url,
                    Err(e) => {
                        warn!("{e}");
                        return None;
                    }
                };
                
                Some(Install {
                    mod_id: mod_id.clone(),
                    mod_name: mod_api.mod_json.name.clone().unwrap_or_default(),
                    version_to_install: mp_mod.version.clone(),
                    download_url,
                    current_file_path: None,
                })
            } else {
                None
            }
        }).collect();
        
        debug!("Need to download {install_mp_mods:#?}");

        let installed = install_manager(&modpack_mod_path, install_mp_mods, HashMap::new()).await?;
        
        debug!("Successfully installed {installed:#?}");
        
        display_table(vec![command_output("Successfully installed Modpack: ".into(), modpack.mod_name)], None);
        elapsed_footer(start_time, "Modpack Install");
    }
    
    Ok(())
}