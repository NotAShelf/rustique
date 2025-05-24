

// Enabling a modpack is a bit of work. First it mvs the existing default mod directory to the rustique backup location,
// then create a symlink to the modpack. Only one modpack can be enabled at a time

use std::path::{Path, PathBuf};
use crate::commands::arg_structs::modpack_args::MPEnableArgs;
use crate::config::config_manager::get_config;
use crate::rustique_errors::RustiqueError;
use crate::traits::ref_ext::PathRef;

pub async fn mp_enable(args: MPEnableArgs, mod_dir: impl PathRef) -> Result<(), RustiqueError> {
    let config = get_config().read().await;
    
    let mod_pack_install_dir = Path::new(&config.modpacks_dir).join("installed").join(args.mpk_id);
    
    if !mod_pack_install_dir.exists() {
        return Err(RustiqueError::SimpleError("Modpack {} doesn't exist. Run 'Rustique modpack list' to view installed modpacks.".into()));
    }
    
    
    
    Ok(())
}