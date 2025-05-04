use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// List only mods that need updating
    #[arg(short, long, default_value = "false")]
    pub(crate) updates: bool
}