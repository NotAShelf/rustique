use std::cmp::Ordering;
use std::str::FromStr;
use tracing::info;
use crate::api::api_structs::{ModApi, ModsSearchFile};
use crate::commands::arg_structs::search_args::SearchArgs;
use crate::commands::sync::{parse_json_file, SEARCH_FILE_NAME};
use crate::config_manager::{get_config, Config};
use crate::rustique_errors::RustiqueError;
use crate::traits::option_ext::OptionExt;
use crate::traits::search_traits::{Searchable, SortValue, Sortable};
use crate::traits::vec_ext::VecStringExt;
//TODO: implement searching by date or time?? Maybe

impl Searchable for ModApi {
    fn matches_text(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.matches_contains(&query)
        || self.summary.matches_contains(&query)
        || self.author.matches_contains(&query)
        || self.mod_type.matches_contains(&query)
        || self.side.matches_contains(&query)
        || self.mod_id_strs.contains(&query)
        || self.url_alias.matches_contains(&query)
        || self.tags.contains(&query)
    }

    fn matches_field(&self, field: &str, value: &str) -> bool {
        match field.to_lowercase().as_str() {
            "name" => self.name.matches_contains(&value),
            "summary" => self.summary.matches_contains(&value),
            "author" => self.author.matches_contains(&value),
            "mod_type" => self.mod_type.matches_contains(&value),
            "side" => self.side.matches_contains(&value),
            "mod_id_strs" => self.mod_id_strs.contains(&value.to_string()),
            "url_alias" => self.url_alias.matches_contains(&value),
            "tags" => self.tags.contains(&value.to_string()),
            _ => false
        }
    }

    fn matches_id(&self, id: u32) -> bool {
        self.mod_id == id || self.asset_id == id
    }

    fn matches_tag(&self, tag: &str) -> bool {
        self.tags.contains_ignore_case(&tag.to_lowercase())
    }
}

impl Sortable for ModApi {
    fn get_sort_value(&self, field: &str) -> SortValue {
        match field.to_lowercase().as_str() {
            "mod_id" | "modid" => SortValue::Number(self.mod_id as i64),
            "asset_id" | "assetid" => SortValue::Number(self.asset_id as i64),
            "downloads" => SortValue::Number(self.downloads as i64),
            "follows" => SortValue::Number(self.follows as i64),
            "trendingpoints" | "trending" => SortValue::Number(self.trending_points as i64),
            "comments" => SortValue::Number(self.comments as i64),
            "name" => SortValue::Text(self.name.clone().unwrap_or_default()),
            "author" => SortValue::Text(self.author.clone().unwrap_or_default()),
            "last_released" | "released" => SortValue::Date(self.last_released.clone().unwrap_or_default()),
            _ => SortValue::Text(String::new())
        }
    }
}

#[derive(Debug, Clone)]
pub enum SearchCriteria {
    Text(String),
    Field {field: String, value: String},
    Id(u32),
    Tag(String),
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Ok(SortOrder::Asc),
            "desc" | "descending" => Ok(SortOrder::Desc),
            _ => Err(format!("Invalid sort order: {}", s))
        }
    }
}

#[derive(Debug)]
pub struct SearchQuery {
    pub criteria: Vec<SearchCriteria>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl SearchQuery {
    pub fn new() -> Self {
        SearchQuery {
            criteria: Vec::new(),
            sort_by: None,
            sort_order: SortOrder::Asc,
        }
    }

    pub fn add_text_search(mut self, text: String) -> Self {
        self.criteria.push(SearchCriteria::Text(text));
        self
    }

    pub fn add_field_search(mut self, field: String, value: String) -> Self {
        self.criteria.push(SearchCriteria::Field {field, value});
        self
    }

    pub fn add_id_search(mut self, id: u32) -> Self {
        self.criteria.push(SearchCriteria::Id(id));
        self
    }

    pub fn add_tag_search(mut self, tag: String) -> Self {
        self.criteria.push(SearchCriteria::Tag(tag));
        self
    }

    pub fn with_sort(mut self, field: String, order: SortOrder) -> Self {
        self.sort_by = Some(field);
        self.sort_order = order;
        self
    }

    pub fn execute(&self, mods: &[ModApi]) -> Vec<ModApi> {
        let mut results: Vec<ModApi> = mods
            .iter()
            .filter(|mod_item| {
                if self.criteria.is_empty() {
                    return true;
                }

                self.criteria.iter().all(|criterion| match criterion {
                    SearchCriteria::Text(query) => mod_item.matches_text(query),
                    SearchCriteria::Field{field, value} => mod_item.matches_field(&field, &value),
                    SearchCriteria::Id(id) => mod_item.matches_id(*id),
                    SearchCriteria::Tag(tag) => mod_item.matches_tag(tag),
                })
            }).cloned().collect();

        if let Some(sort_field) = &self.sort_by {
            results.sort_by(|a, b| {
                let a_val = a.get_sort_value(sort_field);
                let b_val = b.get_sort_value(sort_field);

                let order = a_val.partial_cmp(&b_val).unwrap_or(Ordering::Equal);

                match self.sort_order {
                    SortOrder::Asc => order,
                    SortOrder::Desc => order.reverse(),
                }
            })
        }

        results
    }
}

pub fn parse_search_file() -> Result<ModsSearchFile, RustiqueError> {
    let file_path = Config::get_path().join(SEARCH_FILE_NAME);
    parse_json_file::<ModsSearchFile>(&file_path)
}

pub fn search(args: &SearchArgs) -> Result<(), RustiqueError> {

    let search_file = parse_search_file()?;

    let test = SearchQuery::new().add_text_search(args.query.clone().unwrap_or_default().to_string());

    let res = test.execute(&search_file.mods);

    info!("search result: {:#?}", res);


    Ok(())
}