use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct InstanceCommands {
    #[command(subcommand)]
    pub subcommand: InstanceSubCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum InstanceSubCommands {
    List,
    Show,
    Select(InstanceSelectArgs),
    Upsert(InstanceUpsertArgs),
    Delete(InstanceDeleteArgs),
}

#[derive(Args, Debug, Clone)]
pub struct InstanceSelectArgs {
    pub id: String,
}

#[derive(Args, Debug, Clone)]
pub struct InstanceDeleteArgs {
    pub id: String,
}

#[derive(Args, Debug, Clone)]
pub struct InstanceUpsertArgs {
    pub id: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub mods_dir: String,
    #[arg(long, default_value = "")]
    pub data_dir: String,
    #[arg(long, default_value = "")]
    pub game_version_id: String,
    #[arg(long, default_value = "")]
    pub start_params: String,
    #[arg(long, default_value = "")]
    pub env_vars: String,
}
