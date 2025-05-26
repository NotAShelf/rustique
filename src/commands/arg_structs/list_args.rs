use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// List only mods that need updating
    #[arg(short, long, default_value = "false")]
    pub(crate) updates: bool,
    
    /// List all game versions for MAJOR.MINOR: Example, Rustique list --game-versions 1.20, which will show all valid versions for 1.20.x, --game-versions 1 will show all versions 1.x.x 
    #[arg(short, long, value_name = "MAJOR.MINOR")]
    pub game_versions: Option<String>,
}