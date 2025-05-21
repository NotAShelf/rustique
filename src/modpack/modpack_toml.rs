use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::aliases::ModID;
use crate::rustique_errors::RustiqueError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModPackToml {
    
    pub modpack: ModPack,
    pub mods: HashMap<ModID,MPMods>,
}

impl ModPackToml {
    pub fn save(&self, save_path: &PathBuf, ) -> Result<(), RustiqueError> {
        
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| RustiqueError::SimpleError(format!("Failed in modpack toml save {e}")))?;
        
        
        File::create(save_path)
            .map_err(|e| RustiqueError::SimpleError(format!("Failed to create modpack toml {e}")))?
            .write_all(toml_content.as_bytes())
            .map_err(|e| RustiqueError::SimpleError(format!("Failed to write modpack toml {e}")))?;
        
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ModPack {
    pub name: String,
    pub mpk_id: String,
    pub version: String,
    
    #[serde(default)]
    pub game_version: Option<String>,
    
    #[serde(default)]
    pub description: Option<String>,
    
    #[serde(default)]
    pub author: Option<String>,
    
    #[serde(default)]
    pub contact: Option<String>,
    
    #[serde(default)]
    pub website: Option<String>,
    
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MPMods {
    pub mod_id: ModID,
    pub version: String,
}

