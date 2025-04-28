use std::path::PathBuf;
use colored::Colorize;
use rayon::prelude::*;
use crate::api::ApiClient;
use crate::rustique_errors::RustiqueError;
use crate::utils::{dlog, download_mod, extract_zip_metadata};

pub fn install_mod(mod_dir: &PathBuf, mod_id: &String, ignore_dependencies: bool, api: Option<ApiClient>) -> Result<(), RustiqueError> {

    // get mod_id from api so we have the latest download_url
    let api = api.unwrap_or_else(ApiClient::new);

    let mod_info = api.fetch_mod(mod_id)
        .map_err(|e| RustiqueError::ApiError {
            context: format!("Failed to fetch mod_id: {}", mod_id),
            source: e,
        })?;

    if let Some(download_url) = &mod_info.mod_json.releases[0].main_file {
        // we have the download_url, download the mod into the mods dir
        dlog(&format!("Downloading mod_file: {}", download_url));
        match download_mod(mod_dir, &download_url) {
            Ok(file_path) => {

                eprintln!("{} successfully installed", mod_id.green());

                if !ignore_dependencies {
                    // do dependency check and install
                    let mod_info = extract_zip_metadata(file_path)?;
                    if mod_info.dependencies.is_some() {
                        let mod_ids: Vec<String> = mod_info.dependencies.unwrap().keys()
                            .filter(|k| k.to_lowercase().ne("game"))
                            .cloned()
                            .collect();

                        if mod_ids.len() > 0 {
                            eprintln!("Downloading dependencies {} for {} ...", mod_ids.join(",").green(), mod_id.green());
                            install_mods(mod_dir, mod_ids, ignore_dependencies)?;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to download mod: {}", e.to_string());
            }
        }
    }

    Ok(())
}

pub fn install_mods(mod_dir: &PathBuf, mod_ids: Vec<String>, ignore_dependencies: bool) -> Result<Vec<Result<(), RustiqueError>>, RustiqueError> {
    let api = ApiClient::new();

   let result: Vec<Result<(), RustiqueError>> = mod_ids.par_iter().map(|mod_id| {
       install_mod(mod_dir, mod_id, ignore_dependencies, Some(api.clone()))
   }).collect();

    Ok(result)
}