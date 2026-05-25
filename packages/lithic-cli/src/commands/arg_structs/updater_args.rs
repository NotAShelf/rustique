use clap::Args;

#[derive(Args, Debug)]
pub struct UpdaterArgs {
   /// Manually check if there is a new update for Lithic.
   #[arg(short, long, default_value = "false")]
   pub check_updates: bool,

   /// Update your Lithic binary, if there is one available.
   #[arg(short, long, default_value = "false")]
   pub update: bool,

   /// Force update to the latest version, regardless of current version
   #[arg(short, long, default_value = "false")]
   pub force: bool,
}
