use std::path::PathBuf;
use tracing::error;
use owo_colors::OwoColorize;
use crate::commands::arg_structs::modpack_args::{ModpackCommands, ModpackSubCommands};
use crate::modpack::mp_create::{collect_mp_create_args, mp_create};
use crate::modpack::mp_install::mp_install;

pub async fn parse_modpack_commands(commands: &ModpackCommands, mod_dir: &PathBuf) {
    match &commands.subcommand {
        ModpackSubCommands::Create(args) => {
            let parse_args = collect_mp_create_args(args);
            match mp_create(mod_dir, &mut parse_args.unwrap()).await {
                Ok(_) => {},
                Err(e) => {
                    error!("{}", e.to_string().red().bold());
                }
            }
        }
        ModpackSubCommands::Delete(args) => {}
        ModpackSubCommands::Install(args) => {
            match mp_install(args.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e.to_string().red().bold());
                }
            }
        }
        ModpackSubCommands::Enable(args) => {}
        ModpackSubCommands::Disable(args) => {}
        ModpackSubCommands::List(args) => {}
        ModpackSubCommands::Update(args) => {}
        ModpackSubCommands::Info(args) => {}
    }
}