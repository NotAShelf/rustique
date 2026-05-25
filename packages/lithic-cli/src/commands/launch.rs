use crate::commands::arg_structs::launch_args::LaunchArgs;

pub async fn launch(args: &LaunchArgs) -> Result<(), String> {
    lithic_core::instance::launch_instance(args.instance.clone()).await
}
