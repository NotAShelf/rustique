use comfy_table::{Attribute, Color};
use tracing::{error, info};
use owo_colors::OwoColorize;
use crate::commands::arg_structs::modpack_args::{ModpackCommands, ModpackSubCommands};
use crate::information_utils::notice;
use crate::modpack::mp_create::{collect_mp_create_args, mp_create};
use crate::modpack::mp_enable::mp_enable;
use crate::modpack::mp_install::mp_install;
use crate::traits::ref_ext::PathRef;

pub async fn parse_modpack_commands(commands: &ModpackCommands, mod_dir: impl PathRef) {
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
                    notice("Failed to install modpack. Maybe you have the wrong ID?", Some(Color::Red), vec![Attribute::Bold]);
                    // hide the error for cleaner UX
                    info!("{}", e.to_string().red().bold());
                }
            }
        }
        ModpackSubCommands::Enable(args) => {
            match mp_enable(args.clone(), mod_dir).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e.to_string().red().bold());
                }
            }
        }
        ModpackSubCommands::Disable(args) => {}
        ModpackSubCommands::List(args) => {}
        ModpackSubCommands::Update(args) => {}
        ModpackSubCommands::Info(args) => {}
    }
}