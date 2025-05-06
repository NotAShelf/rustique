use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use url::Url;
use rayon::prelude::*;
use crate::aliases::{DownloadURL, ModID, ModName, ModVersion};
use crate::api::api_structs::Mod;
use crate::api::client::{ApiClient, ModApiFetch};
use crate::api::download::{download_requested_mods};
use crate::commands::sync::ModSyncInfo;
use crate::rustique_errors::RustiqueError;
use crate::utils::extract_zip_metadata;
use crate::version_management::parse_latest_version;

// install & update both will obtain the info needed to fill this struct
#[derive(Debug, Clone)]
pub struct Install {
    pub mod_id: ModID,
    pub mod_name: ModName,
    pub version_to_install: ModVersion,
    pub download_url: DownloadURL,
}


#[derive(Debug, Clone)]
pub struct Installed {
    pub mod_id: ModID,
    pub mod_name: ModName,
    pub install_path: Option<PathBuf>,
    pub success: bool,
}



pub async fn install_manager(
    mod_dir: &PathBuf,
    mods_requested: Vec<Install>,
    installed_mods: HashMap<ModID, ModSyncInfo>) -> Result<(), RustiqueError> {

    // this is the combined list of all mods installed, once download is completed, now mods will be
    // added here
    let mut total_mods_seen: HashMap<ModID, Installed> = HashMap::with_capacity(installed_mods.len());
    installed_mods.iter().for_each(|(mod_id, mod_sync_info)| {
        // this is what is already on the system
        // the version doesn't really matter, we just need to know modid and filepath, which the
        // info from sync would provide that
        total_mods_seen.insert(mod_id.clone(),Installed {
            mod_id: mod_id.clone(),
            mod_name: mod_sync_info.mod_name.clone(),
            install_path: Some(mod_dir.join(mod_sync_info.file_name.clone())),
            success: true,
        });
    });


    info!("total_mods_seen: {:#?}", total_mods_seen);
    info!("mods_requested: {:#?}", mods_requested);

    let client = ApiClient::new();


    // this will be populated again after the dependencies check
    let mut mods_requested = mods_requested.clone();
    let mut mods_installed: Vec<Installed> = Vec::new();

    let mut passes = 0;

    loop {
        let mut recently_installed: Vec<Installed> = Vec::new();


        // this function will consume each value out of the mods_requested so we can rebuild it
        // after the dependencies check
        match download_requested_mods(&mod_dir, &mut mods_requested, &client).await {
            Ok(processed_mods) => {
                info!("Successfully installed mods: {:?}", processed_mods);
                recently_installed.extend(processed_mods);
            }
            Err(err) => {
                error!("Failed to install mods: {:?}", err);
            }
        }

        // add recently seen to total_mods_seen

        for installed in &recently_installed {
            total_mods_seen.insert(installed.mod_id.clone(), installed.clone());
        }


        // extract the modinfojson from recently_installed and gather the dependencies.
        // subtract any dependency which already resides in total seen mods

        let mut needed_dependencies: Vec<Install> = recently_installed.par_iter()
            .filter_map(|installed_mod| {
                let path = installed_mod.install_path.clone()?;
                match extract_zip_metadata(path) {
                    Ok(mod_info) =>  {
                        Some(mod_info.dependencies.filter(|p|{
                            !p.contains_key("game") && !p.contains_key("creative") && !p.contains_key("survival")
                        }).unwrap_or_default())
                    },
                    Err(err) => {
                        error!("Failed to extract zip metadata: {:?}", err);
                        None
                    }
                }
            }).filter(|dep_mod| {
                !total_mods_seen.iter().any(|(mod_id, mod_installed)| {dep_mod.contains_key(mod_id)})


            })
            .flatten()
            .map(|(mod_id, mod_version)| Install {
                mod_id,
                mod_name: "".to_string(),
                version_to_install: mod_version,
                download_url: "".to_string()
            }).collect();


        passes += 1;
        info!("pass: {}, needed_dependencies : {:#?}", passes, needed_dependencies);


        if needed_dependencies.len() == 0 {
            break;
        }

        // obtain the download_urls for the currently needed dependencies and then pass it back to
        // mods_requested

        let mod_ids: Vec<ModID> = needed_dependencies.iter().map(|dep| dep.mod_id.clone()).collect();

        let result: HashMap<ModID, Mod> = client.fetch_mods_parallel(mod_ids).await?;

        info!("Mod api fetch result: {:#?}", result);

        // now add the result to the mods_requested
        // obtain the latest download url
        // and the mod name from the HashMap and update the values in needed_deps
        // then dump needed_deps into requested_mods

        //TODO: double check needed values are present
        for mod_to_install in &mut needed_dependencies {
            if let Some(_mod) =  result.get(mod_to_install.mod_id.as_str()) {
                mod_to_install.mod_name = _mod.mod_json.name.clone().unwrap_or_default();
                let (mod_version, download_url) = parse_latest_version(&_mod.mod_json.releases);
                mod_to_install.download_url = download_url;
                mod_to_install.version_to_install = mod_version;
            }
        }
        // seed the mods_requested and go again
        mods_requested.extend(needed_dependencies);
    }


    Ok(())

}