use clap::{Args, Subcommand, ValueEnum};
use lithic_core::api::client::{VSExecutabletype, VSOSType, VSWinInstallerType};

#[derive(Args, Debug, Clone)]
pub struct GameVersionCommands {
    #[command(subcommand)]
    pub subcommand: GameVersionSubCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GameVersionSubCommands {
    List,
    Add(GameVersionAddArgs),
    Install(GameVersionInstallArgs),
    Remove(GameVersionRemoveArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GameVersionAddArgs {
    pub id: String,
    #[arg(long)]
    pub version: String,
    #[arg(long)]
    pub path: String,
    #[arg(long, value_enum, default_value = "manual")]
    pub source: GameVersionSourceArg,
}

#[derive(Args, Debug, Clone)]
pub struct GameVersionRemoveArgs {
    pub id: String,
}

#[derive(Args, Debug, Clone)]
pub struct GameVersionInstallArgs {
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub version: String,
    #[arg(long)]
    pub install_dir: Option<String>,
    #[arg(short, long, value_name = "OS", default_value_t = os_default())]
    pub os_type: VSOSType,
    #[arg(
        short = 't',
        long = "type",
        value_name = "TYPE",
        default_value = "client"
    )]
    pub exe_type: VSExecutabletype,
    #[arg(short, long, default_value = "install")]
    pub windows_installer_type: Option<VSWinInstallerType>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum GameVersionSourceArg {
    Manual,
    LithicDownload,
}

fn os_default() -> VSOSType {
    #[cfg(target_os = "macos")]
    return VSOSType::OSX;
    #[cfg(target_os = "windows")]
    return VSOSType::Windows;
    #[cfg(target_os = "linux")]
    VSOSType::Linux
}
