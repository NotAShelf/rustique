use clap::{Args, Parser, Subcommand};
use crate::modpack_commands::*;
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {

    #[arg(short, long)]
    pub(crate) mods_dir: Option<String>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Checks with the VintageStory mods website for any updates to mods you have installed. Run update after this command to update your mods")]
    Sync(SyncArgs),

    #[command(about = "List installed mods and their versions. Run sync first to show latest version of the mod.")]
    List(ListArgs),

    #[command(about = "Updates a specific mod OR all mods installed. Runs sync after completion")]
    Update(UpdateArgs),

    #[command(about = "View the changelogs for a installed mod")]
    Changelog(ChangeLogArgs),

    #[command(about = "Install a specific mod. Must use the mod_id, Example: ./Rustique install alchemy")]
    Install(InstallArgs),

    #[command(about = "Shows values from the modinfo.json file inside the mod zip")]
    Info(ModInfoArgs),

    #[command(about = "Search the mob website for mobs.")]
    Search(SearchMods),

    ModPack {
        #[clap(subcommand)]
        command: ModpackCommands,
    },
}

#[derive(Args)]
pub struct SyncArgs {
}

#[derive(Args)]
pub struct ListArgs {
    /// List only mods that need updating
    #[arg(short, long, default_value = "false")]
    pub(crate) updates: bool
}

#[derive(Args)]
pub struct UpdateArgs {

    /// Update specific mod, must be mod_id. Example: ./Rustique update alchemy
    pub(crate) mod_ids: Vec<String>,

    /// Update all mods, don't set a <name>. Example: ./Rustique update --all
    #[arg(short, long)]
    pub(crate) all: bool,

    /// Update mods but keep old version.
    #[arg(short, long)]
    pub(crate) keep_old_files: bool
}

#[derive(Args)]
pub struct ChangeLogArgs {
    pub(crate) name: Option<String>,
}

#[derive(Args)]
pub struct InstallArgs {
    #[arg(num_args = 1..)]
    pub(crate) mod_ids: Vec<String>,

    #[arg(short, long, default_value = "false")]
    pub(crate) ignore_dependencies: bool,
}

#[derive(Args)]
pub struct ModInfoArgs {
    pub(crate) mod_id: String,
}

#[derive(Args)]
pub struct SearchMods {

}
