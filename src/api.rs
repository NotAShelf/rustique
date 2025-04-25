

pub mod api {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use ureq::{Agent, Error};
    use rayon::prelude::*;
    use ureq::config::Config;
    use crate::api_structs::{Mod, ModInfo, Mods};

    const API_BASE_URL: &str = "http://mods.vintagestory.at/api";
    const RUSTIQUE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"), "  (github: Tekunogosu/Rustique)");

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

        pub fn fetch_all_mods(&self) -> Result<Mods, Error> {
            self.agent.get(&self.uri("mods")).call()?.body_mut().read_json::<Mods>()
        }

        pub fn fetch_mod(&self, mod_id: &str) -> Result<Mod, Error> {
            self.agent.get(&self.uri(&format!("mod/{}", mod_id))).call()?.body_mut().read_json::<Mod>()
        }

        // pub fn fetch_mods_parallels(&self, mod_list: Vec<ModInfo>) -> Vec<Result<Mod, String>> {
        //     let client = Arc::new(self);
        //
        //     mod_list.par_iter()
        //         .map(|mod_info| {
        //             let agent = client.clone();
        //             // print!("Attempting api call for {}", mod_info.mod_id);
        //             agent.fetch_mod(mod_info.mod_id.as_ref())
        //                 .map_err(|err| {
        //                     // println!("error: {:?}", err);
        //                     err.to_string()
        //                 })
        //         }).collect()
        // }

         pub fn fetch_mods_parallel(&self, mod_list: Vec<ModInfo>) -> Result<HashMap<String, Mod>, String> {
            let client = Arc::new(self);

             mod_list.par_iter()

                 .map(|mod_info| {
                     let agent = client.clone();
                     // print!("Attempting api call for {}", mod_info.mod_id);
                     match agent.fetch_mod(mod_info.mod_id.as_ref()) {
                         Ok(the_mod) => Ok((mod_info.mod_id.clone(), the_mod)),
                         Err(err) => Err(err.to_string())
                     }
                 }).collect()
        }
    }
}