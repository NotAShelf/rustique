use clap::{Args, Subcommand,ArgGroup};

#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub(crate) subcommand: ConfigSubCommand,
}

#[derive(Subcommand)]
pub enum ConfigSubCommand {

    /// Set a value in the config file
    Set(SetArgs),

    /// List all config options and their current values
    List,

    /// Show a specific option and its value
    Show(ShowArgs),

    /// Deletes an option, returning it to the default value
    Del(DelArgs),
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("common_args_flags")
    .args([
        "mods_dir",
        "pin_game_version",
        "backup_mods",
        "backup_mods_dir",
        "zip_mod_dirs",
        "show_mod_dir_warning"
    ]).required(false)
))]
pub struct CommonArgs {

    /// Default mod directory Rustique will manage
    ///
    /// This path MUST be an absolute path
    ///
    /// Example: /home/username/.config/VintagestoryData/Mods
    ///
    /// You can use ~/ as well, it will expand into /home/username/
    ///
    /// Default: '~/.config/VintagestoryData/Mods' for Unix systems (Linux and Mac)
    ///          '%appdata%/Vintagestory/Mods' for windows
    #[arg(short, long)]
    pub mods_dir: Option<String>,

    #[arg(short, long)]
    pub show_mod_dir_warning: Option<bool>,

    /// The highest game version Rustique will use to download mods
    #[arg(short, long, value_name = "GAME_VERSION")]
    pub pin_game_version: Option<String>,

    /// Backup your mods before updating, preserves older versions
    #[arg(short, long, default_value = "false")]
    pub backup_mods: Option<bool>,

    /// Directory for mod backups
    #[arg(short = 'B', long, value_name = "DIR")]
    pub backup_mods_dir: Option<String>,

    /// Rustique will attempt to identify mods that are not zipped and zip them for you.
    #[arg(short, long, default_value = "false")]
    pub zip_mod_dirs: Option<bool>,
}

#[derive(Args, Debug)]
pub struct SetArgs {
    #[command(flatten)]
    pub common: CommonArgs
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    #[arg(required = true)]
    pub key: String,
    #[command(flatten)]
    pub common: CommonArgs
}

#[derive(Args, Debug)]
pub struct DelArgs {

    #[arg(short, long)]
    pub mods_dir: bool,

    /// The highest game version Rustique will use to download mods
    #[arg(short, long)]
    pub pin_game_version: bool,

    /// Backup your mods before updating, preserves older versions
    #[arg(short, long)]
    pub backup_mods: bool,

    /// Directory for mod backups
    #[arg(short = 'B', long)]
    pub backup_mods_dir: bool,

    /// Rustique will attempt to identify mods that are not zipped and zip them for you.
    #[arg(short, long)]
    pub zip_mod_dirs: bool,
}

