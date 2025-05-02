use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ureq::{Agent, Body, Error};
use rayon::prelude::*;
use ureq::config::Config;
use ureq::http::Response;
use crate::api_structs::{Mod, ModInfo, Mods};
use crate::rustique_errors::RustiqueError;

const API_BASE_URL: &str = "https://mods.vintagestory.at/api";
const RUSTIQUE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "  (github: Tekunogosu/Rustique)");

#[derive(Debug, Clone)]
pub struct ApiClient {
    agent: Arc<Agent>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            agent: Arc::new(
                Agent::new_with_config(
                    Config::builder()
                        .timeout_global(Some(Duration::from_secs(20)))
                        .user_agent(RUSTIQUE_USER_AGENT)
                        .build()
                )
            ),
        }
    }

    pub fn with_agent(agent: Arc<Agent>) -> Self {
        Self { agent }
    }

    fn uri(&self, endpoint: &str) -> String {
        format!("{}/{}", API_BASE_URL, endpoint)
    }

    pub fn fetch_all_mods(&self) -> Result<Mods, RustiqueError> {
        self.agent.get(&self.uri("mods")).call().map_err(|e| RustiqueError::ApiError {
            context: "fetch_all_mods (get): ".to_string(),
            source: e,
        })?.body_mut().read_json::<Mods>().map_err(|e| RustiqueError::ApiError {
            context: "fetch_all_mods (json): ".to_string(),
            source: e,
        })
    }

    pub fn fetch_mod(&self, mod_id: &str) -> Result<Mod, RustiqueError> {
        self.agent.get(&self.uri(&format!("mod/{}", mod_id))).call().map_err(|e| RustiqueError::ApiError {
            context: format!("fetch_mod (get) [{}]", mod_id),
            source: e
        })?.body_mut().read_json::<Mod>().map_err(|e| RustiqueError::ApiError {
            context: format!("fetch_mod (json) [{}]", mod_id),
            source: e
        })
    }

    pub fn fetch_mods_parallel(&self, mod_list: Vec<ModInfo>) -> Result<HashMap<String, Mod>, RustiqueError> {
        let client = Arc::new(self);

         let result = mod_list
             .par_iter()
             .filter_map(|mod_info| {
                 let agent = client.clone();
                 match agent.fetch_mod(mod_info.mod_id.as_ref()) {
                     Ok(the_mod) => {
                         Some((mod_info.mod_id.clone(), the_mod))
                     },
                     Err(e) => {
                         eprintln!("{} {}", mod_info.mod_id, e);
                         None
                     }
                 }
             }).collect();

        Ok(result)
    }

    pub fn get_request(&self, mod_uri: &str) -> Result<Response<Body>, RustiqueError> {
        self.agent.get(mod_uri).call().map_err(|e| RustiqueError::ApiError {
            context: format!("get_request: {}",mod_uri),
            source: e,
        })
    }
}
