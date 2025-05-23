
// mp_install calls normal install, except it gathers a list of all the required mods before hand 
// and sets the install location to be the modpack specific folder

use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info};
use crate::api::client::ApiClient;
use crate::commands::arg_structs::modpack_args::{MPCreateArgs, MPInstallArgs};
use crate::commands::install::install_cmd;
use crate::commands::search::{parse_search_file, SearchQuery};
use crate::config::config_manager::get_config;
use crate::install_manager::{install_manager, Install};
use crate::rustique_errors::RustiqueError;
use crate::version_management::parse_latest_version;

pub async fn mp_install(args: MPInstallArgs) -> Result<(), RustiqueError> {
    // installing the modpack with this function will do the following:
    // Save the modpack.zip (the modpack from the mods website) to modpacks/packs
    // Once the modpack is installed, it will download all the mods associated with the modpack  
    // to the location [modpacks/installed/modpack_id/*] 
    let config = get_config().read().await;
   
    let client = ApiClient::new();
    
    let mod_info = client.fetch_mod(&args.mod_id).await?;
    
    let modpack_path = Path::new(&config.modpacks_dir).join("installed").join(&args.mod_id);
    
    if !modpack_path.exists() {
        info!("Created {modpack_path:?}");
        std::fs::create_dir_all(&modpack_path)?;
    }
    
    let (version, download_url, _) = parse_latest_version(&mod_info.mod_json.releases);
    
    let install_modpack = Install {
        mod_id: mod_info.mod_json.mod_id.clone().to_string(),
        mod_name: mod_info.mod_json.name.clone().unwrap_or_default(),
        version_to_install: version,
        download_url,
        current_file_path: None,
    };
    
    // each modpack will be a new folder, so the mods_installed will always be a new hashmap as there is nothing there
    match install_manager(&modpack_path, vec![install_modpack], HashMap::new()).await {
        Ok(installed) => {
            println!("installed {installed:?}");
        }
        Err(e) => {
            error!("failed to install modpack {}: {}", &args.mod_id, e.to_string());
        }
    }
    
    
    Ok(())
}