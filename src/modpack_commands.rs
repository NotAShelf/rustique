use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum ModpackCommands {
    Create(MPCreateArgs)
}

#[derive(Args)]
pub struct MPCreateArgs {
    pub(crate) name: String,
    pub(crate) mod_dir: Option<String>,
}