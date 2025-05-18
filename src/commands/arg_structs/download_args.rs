use crate::utils::latest_stable;
use clap::ArgGroup;
use clap::Args;
use crate::api::client::{VSExecutabletype, VSOSType};
use crate::rustique_errors::RustiqueError;

#[derive(Args, Debug)]
pub struct DownloadArgs {

    /// Set where you want to save the download, will use your default mod_dir if omitted 
    #[arg(short, long, value_name = "PATH")]
    pub save_dir: Option<String>,
    
    /// To see valid game versions use `Rustique list --game-versions`, default is latest stable version
    #[arg(short, long, value_name = "VERSION", default_value_t = latest_stable())]
    pub game_version: String,
    
    /// Choose the os type to download
    #[arg(short, long, value_name = "OS", default_value_t = os_default())]
    pub os_type: VSOSType,
    
    
    /// Select executable type, client by default. Note: Mac `DOES NOT` have a server
    #[arg(short = 't', long = "type",  value_name = "TYPE", default_value = "client")]
    pub exe_type: VSExecutabletype
}

impl DownloadArgs {
    pub fn validate(&self) -> Result<(), RustiqueError> {
        match (&self.exe_type, &self.os_type) {
            (VSExecutabletype::Server, VSOSType::OSX) => Err(RustiqueError::SimpleError("Server type is not available for Mac".into())),
            _ => Ok(()),
        }
    }
}

fn os_default() -> VSOSType {
    #[cfg(target_os = "macos")]
    return VSOSType::OSX;
    #[cfg(target_os = "windows")]
    return VSOSType::Windows;
    #[cfg(target_os = "linux")]
    VSOSType::Linux
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("update_or_install")
        .args(["install", "update"])
        .multiple(false)
        .required(true)
))]
pub struct UpdateOrInstall {
    pub update: bool,
    pub install: bool,
}

