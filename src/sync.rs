use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::utils::{RustiqueOptions, get_current_time, extract_all_mods_metadata};
use crate::api::api::ApiClient;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde_json::to_string_pretty;
use crate::api_structs::{Mod, ModInfo};
use ureq::Agent;

#[derive(Deserialize, Serialize, Debug)]
pub struct RustiqueSyncJson {
    #[serde(rename = "RustiqueSync")]
    pub rustique_sync: HashMap<String, ModSyncInfo>,
    pub last_sync: String,
}

impl RustiqueSyncJson {
    pub fn new() -> RustiqueSyncJson {
        Self {
            rustique_sync: HashMap::<String, ModSyncInfo>::new(),
            last_sync: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ModSyncInfo {
    pub file_name: String,
    pub installed_version: String,
    pub latest_known_version: String,
    pub latest_download_url: String,
}

pub const SYNC_FILE_NAME: &str = "rustique-sync.json";


fn parse_sync_file(dir: PathBuf) -> Result<RustiqueSyncJson, Box<dyn Error>> {
    let file = File::open(dir.join(SYNC_FILE_NAME))?;
    let mut file_contents = String::new();
    let json = serde_json::from_str::<RustiqueSyncJson>(&file_contents)?;

    Ok(json)
}

pub fn sync(rustique_opts: RustiqueOptions) -> Result<(),Box<dyn Error>> {

    // check if rustique-sync.json exists
    // if so, parse the file for updating
    // if not, do all the sync process and then write a new file

    let file_path = rustique_opts.mod_dir.as_ref().unwrap().join(SYNC_FILE_NAME);

    println!("rustique-sync.json: {}", file_path.display());

    let sync_data = if file_path.exists() {
       parse_sync_file(file_path.clone())?
    } else {
        RustiqueSyncJson {
            rustique_sync: HashMap::new(),
            last_sync: get_current_time(),
        }
    };

    // wrap the sync_data in an arc/mutex for our threads
    // mut isn't required as Mutex defines that internally
    let sync_data = Arc::new(Mutex::new(sync_data));


    let agent = Arc::new(
        Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
    );



    // the filtermap lets us grab the valid results from the extract function without having to use match
    let installed_mods= extract_all_mods_metadata(rustique_opts)
        .map_err(|e| e.to_string())?;

    // installed_mods.par_iter().map(|mod_info| {
    //     let agent = agent.clone();
    //
    // }).collect();

    let result: Vec<Mod> = ApiClient::new()
        .fetch_mods_parallel(installed_mods)
        .into_iter()
        .filter_map(Result::ok).collect();

    result.iter().for_each(|mod_info| {

        let installed_version = "Needs fixed";
        let file_name = "Needs fixed";
        let latest_known_version = &mod_info.mod_json.releases[0].mod_version;
        let latest_download_url = &mod_info.mod_json.releases[0].main_file;

        sync_data.lock().unwrap()
            .rustique_sync
            .entry(mod_info.mod_json.name.as_ref().unwrap_or(&String::new()).clone())
            .or_insert_with(|| ModSyncInfo {
                installed_version: installed_version.to_string(),
                file_name: file_name.to_string(),
                latest_known_version: latest_known_version.as_ref().unwrap().to_string(),
                latest_download_url: latest_download_url.as_ref().unwrap().to_string(),
            });
    });

    println!("Sync complete at {}", sync_data.lock().unwrap().last_sync);
    sync_data.lock().unwrap().rustique_sync.iter().for_each(|(mod_id, mod_json)| {
        println!("{}: ", mod_id);
        println!("\t{:#?}", mod_json);
    });

    let data = sync_data.lock().unwrap();
    let json = to_string_pretty(&*data)?;
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;


    Ok(())
}

