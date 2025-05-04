use std::path::PathBuf;
use tracing::warn;
use colored::Colorize;
use crate::commands::arg_structs::config_args::{CommonArgs, ConfigCommand, ConfigSubCommand, DelArgs, SetArgs, ShowArgs};
use crate::commands::arg_structs::list_args::ListArgs;
use crate::config_manager::get_config;
use crate::utils::get_expanded_path;

pub fn parse_config_args(config_cmd: &ConfigCommand) {
    match &config_cmd.subcommand {
        ConfigSubCommand::Set(args) => {
            set(&args)
        },
        ConfigSubCommand::List => {
            println!("listing all configurations");
        },
        ConfigSubCommand::Show(args) => {
            println!("{:?}", args);
        },
        ConfigSubCommand::Del(args) => {
            println!("{:?}", args);
        },
    }
}


fn set(args: &SetArgs) {
    let mut config = get_config().write().unwrap();

    match &args.common {
        CommonArgs { mods_dir: Some(path), ..} => {
            let dir = get_expanded_path(PathBuf::from(path));
            if dir.exists() {
                config.mod_dir = dir.to_string_lossy().to_string();
                config.save(None).unwrap();
                eprintln!("{} set to {}", "config.mods_dir".bright_green().bold(), dir.to_string_lossy().bright_magenta().bold());
            } else {
                warn!("{} is not a valid directory", dir.to_string_lossy());
            }
        }
        CommonArgs { pin_game_version: Some(version), ..} => {
            config.pinned_game_version = version.to_string();
            config.save(None).unwrap();
            eprintln!("{} set to {}", "config.pinned_game_version".bright_green().bold(), config.pinned_game_version);
        }
        CommonArgs { zip_mod_dirs: Some(zip_it), .. } => {
        }
        CommonArgs { backup_mods: Some(backup), .. } => {}
        CommonArgs { backup_mods_dir: Some(backup_dir), ..} => {}

        _ => {}
    }
}

fn show(args: &ShowArgs) {
    match &args.common {
        CommonArgs { mods_dir, ..} => {}
        CommonArgs { pin_game_version: String, ..} => {}
        CommonArgs { zip_mod_dirs: bool, .. } => {}
        CommonArgs { backup_mods: bool, .. } => {}
        CommonArgs { backup_mods_dir: String, ..} => {}
        _ => {}
    }
}

fn list() {
    println!("listing all configurations...");
}

fn del(args: &DelArgs) {
   match &args {
       DelArgs { mods_dir: true, .. } => {
           println!("Setting mods_dir to default");
       }
       DelArgs { pin_game_version, .. } => {
           println!("Setting pin_game_version to default");
       }
       DelArgs { backup_mods: true, .. } => {
           println!("Setting backup_mods to default");
       }
       DelArgs { backup_mods_dir: true, .. } => {
           println!("Setting backup_mods to default");
       }
       DelArgs { zip_mod_dirs: true, .. } => {
           println!("Setting zip_mods to default");
       }

       _ => {}
   }
}
