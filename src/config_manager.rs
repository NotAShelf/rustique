use crate::rustique_errors::RustiqueError;
use crate::utils::{notice, rustique_message, CellData, RustiqueMessage, RustiqueOptions};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::thread::current;
use chrono::Local;
use comfy_table::{Attribute, CellAlignment, Color, Column};
use dirs::home_dir;
use tracing::error;
use crate::config_structs::{AliasConfig, ModPack, TableSection, Tables};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    // this sets the default mod dir so you don't have to type -m everytime
    pub mod_dir: String,
    // this tells rustique which versions of the game to download mods for.
    // It will download mods up to this version and not over
    pub pinned_game_version: String,
    // automatically zips mod folders that are unzipped during the sync process
    pub zip_mod_files: bool,
    // create a backup of each mod before its updated.
    pub backup_mods: bool,

    // location for the mod backups
    // default ~/.config/rustique/backups
    pub backup_mods_dir: String,

    // Shows the "<operation> completed: " text after a command finishes
    pub show_execution_time: bool,

    pub notify_of_unzipped_mods: bool,

    pub mod_pack: ModPack,
    pub alias: Vec<AliasConfig>,
    pub table: Tables,
}

impl Config {
    pub fn get_path() -> PathBuf {
        if cfg!(target_os = "windows") {
            if let Some(w_path) = std::env::var_os("APPDATA") {
                PathBuf::from(w_path).join("rustique")
            } else {
                PathBuf::from(".").join("rustique")
            }
        } else {
            if let Some(u_path) = home_dir() {
                u_path.join(".config").join("rustique")
            } else {
                PathBuf::from(".").join("rustique")
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // let backup_mods_dir = get_expanded_path(PathBuf::from(CONFIG_DEFAULT_DIR).join("mod_backups"));
        let backup_mods_dir = Self::get_path().join("mod_backups");
        Self {
            mod_dir: RustiqueOptions::default().mod_dir.unwrap().to_string_lossy().to_string(),
            pinned_game_version: "".to_string(), // if its empty then get the latest
            zip_mod_files: false,
            backup_mods: false,
            backup_mods_dir: backup_mods_dir.to_string_lossy().to_string(),
            show_execution_time: true,
            mod_pack: ModPack {},
            notify_of_unzipped_mods: false,
            alias: vec![],
            table: Tables::with_defaults()
        }
    }
}

impl Config {
    pub fn new(config_dir: Option<PathBuf>) -> Result<Config, RustiqueError> {

        let config_path = config_dir.unwrap_or_else(|| Self::get_path());

        if !config_path.exists() {
            fs::create_dir_all(&config_path).map_err(|e| {
                RustiqueError::ConfigFileError(format!("Failed to create config directory: {}", e.to_string()))
            })?;
        }

        let config_file_path = config_path.join("config.toml");

        if !config_file_path.exists() {
            let default_config = Self::default();
            let toml_content = toml::to_string_pretty(&default_config).map_err(|e| {
                RustiqueError::ConfigFileError(format!("Failed to serialize default config: {}", e.to_string()))
            })?;

            let mut file = File::create(&config_file_path).map_err(|e| {
                RustiqueError::ConfigFileError(format!("Failed to create config file at: {}", e.to_string()))
            })?;

            file.write_all(toml_content.as_bytes()).map_err(|err| {
                RustiqueError::ConfigFileError(format!("Failed writing config file: {}", err.to_string()))
            })?;

            println!("{} {}","Successfully created config file: ".green(), config_file_path.display().to_string().bright_yellow());
            return Ok(default_config);
        };

        // if config exists load and parse it
        let mut file = File::open(&config_file_path)
            .map_err(|e| RustiqueError::ConfigFileError(format!("Failed to open config file: {}", e.to_string())))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            RustiqueError::ConfigFileError(format!("Failed to read config file: {}", e.to_string()))
        })?;

        match toml::from_str::<Config>(&contents) {
            Ok(config) => Ok(config),
            Err(e) => {
                // eprintln!("Failed to parse config: {}", e.to_string().red());
                // eprintln!("Using default config");
                backup_config(&config_file_path, Some(e.to_string()))?;

                // write the default

                let config = Config::default();
                config.save(Option::from(Config::get_path()))?;

                Ok(config)
            }
        }
    }

    pub fn save(&self, config_dir: Option<PathBuf>) -> Result<(), RustiqueError> {
        let config_path = config_dir.unwrap_or_else(|| Self::get_path());
        let config_file_path  = config_path.join("config.toml");

        let toml_content = toml::to_string_pretty(self).map_err(|e| {
            RustiqueError::ConfigFileError(format!("Failed to serialize config: {}", e.to_string()))
        })?;

        File::create(&config_file_path)
            .map_err(|e| RustiqueError::ConfigFileError(format!("Failed to create config file: {}", e.to_string())))?
            .write_all(toml_content.as_bytes())
            .map_err(|e| RustiqueError::ConfigFileError(format!("Failed to write config file: {}", e.to_string())))?;

        Ok(())
    }
}

pub fn backup_config(config_path: &PathBuf, message: Option<String>) -> Result<(), RustiqueError> {
    if config_path.exists() {
        let back_name = format!("toml.bak-{}", Local::now().format("%Y%m%d_%H%M%S"));
        let backup_path = config_path.with_extension(&back_name);

        let h1 = CellData::new(
            "Rustique has discovered an error with your config.toml file".to_string(),
            Some(Color::Magenta),
            vec![Attribute::Bold],
            None,
        );

        let m1 = CellData::new(
            "Your old config has been backed up to the following location:".to_string(),
            Some(Color::Yellow), vec![], None
        );

        let m2 = CellData::new(
            format!("{}", config_path.with_extension(&back_name).display()),
            Some(Color::Green),
            vec![Attribute::Bold],None,
        );

        let m3 = CellData::new(
          "A new config has been written using default values. You will need to set your configuration options again.".to_string(),
          Some(Color::Yellow),
          vec![],None,
        );

        let m4 = CellData::new("".to_string(), None, vec![], None);
        let m5 = CellData::new(
            format!("{}", message.unwrap_or_default()),
            Some(Color::Red),
            vec![Attribute::Bold, Attribute::Italic],
            Some(CellAlignment::Left)
        );

        rustique_message(RustiqueMessage {
            header: Some(h1),
            message: vec![m1,m2,m3, m4, m5],
        });

        fs::copy(config_path, &backup_path)?;
    }

    Ok(())
}

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

// Initiate the CONFIG in the main file so its ready everywhere else
pub fn init_config(config_dir: Option<PathBuf>) -> Result<(), RustiqueError> {
    let config = Config::new(config_dir)?;

    if CONFIG.set(RwLock::new(config)).is_err() {
        return Err(RustiqueError::ConfigFileError("Config has already been initialized".to_string()));
    }

    Ok(())
}

pub fn get_config() -> &'static RwLock<Config> {
    CONFIG.get_or_init(|| RwLock::new(Config::new(None).expect("Config has not been initialized")))
}