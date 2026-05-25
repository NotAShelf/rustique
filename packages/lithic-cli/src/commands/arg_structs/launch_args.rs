use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct LaunchArgs {
   #[arg(long)]
   pub instance: Option<String>,
}
