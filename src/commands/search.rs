
use std::cmp::Ordering;
use std::str::FromStr;
use clap::ValueEnum;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Row, Table};
use owo_colors::OwoColorize;
use tracing::{debug, info};
use crate::api::api_structs::{ModApi, ModsSearchFile};
use crate::commands::arg_structs::search_args::SearchArgs;
use crate::commands::sync::{parse_json_file, SEARCH_FILE_NAME};
use crate::config_manager::{get_config, Config};
use crate::config_structs::{CellColor, SearchColumn};
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

    fn matches_field(&self, field: &Field, value: &str) -> bool {
        match field {
            Field::Name     => self.name.matches_contains(&value),
            Field::Summary  => self.summary.matches_contains(&value),
            Field::Author   => self.author.matches_contains(&value),
            Field::ModType  => self.mod_type.matches_contains(&value),
            Field::Side     => self.side.matches_contains(&value),
            Field::ModIdStr => self.mod_id_strs.contains(&value.to_string()),
            Field::UrlAlias => self.url_alias.matches_contains(&value),
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

#[derive(Debug, Clone, ValueEnum)]
pub enum Field {
    Name,
    Summary,
    Author,
    ModType,
    Side,
    ModIdStr,
    UrlAlias,
    Tags
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SortBy {
    ModId,
    AssetId,
    Downloads,
    Follows,
    Trending,
    Comments,
    Name,
    Author,
    Released,
}

impl Sortable for ModApi {
      fn get_sort_by(&self, field: &SortBy) -> SortValue {
        match *field {
            SortBy::Name        => SortValue::Number(self.mod_id as i64),
            SortBy::AssetId     => SortValue::Number(self.asset_id as i64),
            SortBy::Downloads   => SortValue::Number(self.downloads as i64),
            SortBy::Follows     => SortValue::Number(self.follows as i64),
            SortBy::Author      => SortValue::Number(self.trending_points as i64),
            SortBy::Released    => SortValue::Number(self.comments as i64),
            SortBy::Comments    => SortValue::Text(self.name.clone().unwrap_or_default()),
            SortBy::Trending    => SortValue::Text(self.author.clone().unwrap_or_default()),
            SortBy::ModId       => SortValue::Date(self.last_released.clone().unwrap_or_default()),
        }
    }
}



#[derive(Debug, Clone)]
pub enum SearchCriteria {
    Text(String),
    Field {field: Field, value: String},
    Id(u32),
    Tag(String),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
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

#[allow(dead_code, unused)]
#[derive(Debug)]
pub struct SearchQuery {
    pub criteria: Vec<SearchCriteria>,
    pub sort_by: Option<SortBy>,
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

    pub fn add_field_search(mut self, field: Field, value: String) -> Self {
        self.criteria.push(SearchCriteria::Field {field, value});
        self
    }

    #[allow(dead_code)]
    pub fn add_id_search(mut self, id: u32) -> Self {
        self.criteria.push(SearchCriteria::Id(id));
        self
    }

    pub fn add_tag_search(mut self, tag: String) -> Self {
        self.criteria.push(SearchCriteria::Tag(tag));
        self
    }

    pub fn sort_by(mut self, field: SortBy) -> Self {
        self.sort_by = Some(field);
        self
    }

    pub fn with_sort(mut self, field: SortBy, order: SortOrder) -> Self {
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
                let a_val = a.get_sort_by(&sort_field);
                let b_val = b.get_sort_by(&sort_field);

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

    let mut query = SearchQuery::new();

    if args.field.is_some() && args.query.is_some() {
        query = query.add_field_search(args.field.clone().unwrap_or(Field::Summary), args.query.clone().unwrap_or_default());
    } else if args.query.is_some() {
        query = query.add_text_search(args.query.clone().unwrap_or_default());
    }


    if args.author.is_some() {
        query = query.add_field_search(Field::Author, args.author.clone().unwrap_or_default());
    }

    if args.tag.is_some() {
        query = query.add_tag_search(args.tag.clone().unwrap_or_default());
    }


    if args.sort_direction.is_some() {
        query.sort_order = args.sort_direction.unwrap_or(SortOrder::Asc);
    }

    if args.sort_by.is_some() {
        query.sort_by = Some(args.sort_by.clone().unwrap_or(SortBy::Name));
    }

    let res = query.execute(&search_file.mods);

    debug!("search result: {:#?}", res);

    show_search_table(res);


    Ok(())
}

pub fn show_search_table(results: Vec<ModApi>) {
    let config = get_config().read().unwrap();

    let search_config = &config.table.search;
    let search_headers = &search_config.headers;
    let search_cells = &search_config.cells;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let mut headers = Row::new();

    for (k, v) in search_headers.iter() {
        let color = v.color.clone();
        let attr = v.attribute.clone();

        match SearchColumn::from_str(k) {
            Ok(SearchColumn::Name) => {
                headers.add_cell(prep_cell("Name", color, attr));
            }
            Ok(SearchColumn::Author) => {
                headers.add_cell(prep_cell("Author", color, attr));
            }
            Ok(SearchColumn::ModId) => {
                headers.add_cell(prep_cell("Mod ID", color, attr));
            }
            Ok(SearchColumn::ModidStrs) => {
                headers.add_cell(prep_cell("ModID Strings", color, attr));
            }
            Ok(SearchColumn::AssetId) => {
                headers.add_cell(prep_cell("Asset ID", color, attr));
            }
            Ok(SearchColumn::Downloads) => {
                headers.add_cell(prep_cell("Downloads", color, attr));
            }
            Ok(SearchColumn::Follows) => {
                headers.add_cell(prep_cell("Follows", color, attr));
            }
            Ok(SearchColumn::Trending) => {
                headers.add_cell(prep_cell("Trending", color, attr));
            }
            Ok(SearchColumn::Comments) => {
                headers.add_cell(prep_cell("Comments", color, attr));
            }
            Ok(SearchColumn::Summary) => {
                headers.add_cell(prep_cell("Summary", color, attr));
            }
            Ok(SearchColumn::UrlAliases) => {
                headers.add_cell(prep_cell("Url Aliases", color, attr));
            }
            Ok(SearchColumn::Side) => {
                headers.add_cell(prep_cell("Side", color, attr));
            }
            Ok(SearchColumn::Type) => {
                headers.add_cell(prep_cell("Type", color, attr));
            }
            Ok(SearchColumn::Tags) => {
                headers.add_cell(prep_cell("Tags", color, attr));
            }
            Ok(SearchColumn::LastReleased) => {
                headers.add_cell(prep_cell("Last Released", color, attr));
            }
            _ => {
                headers.add_cell(prep_cell("N/A", None, None));
            }
        }
    }

    let b_rows: Vec<Row> = results.iter().map(|m| {
        let cells: Vec<Cell> = search_cells.iter().map(|(k,v)| {
            let color = v.color.clone();
            let attr = v.attribute.clone();

            match SearchColumn::from_str(k) {
                Ok(SearchColumn::Name) => prep_cell(m.name.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::ModId) => prep_cell(m.mod_id.to_string().as_str(), color, attr),
                Ok(SearchColumn::AssetId) => prep_cell(m.asset_id.to_string().as_str(), color, attr),
                Ok(SearchColumn::Downloads) => prep_cell(m.downloads.to_string().as_str(), color, attr),
                Ok(SearchColumn::Follows) => prep_cell(m.follows.to_string().as_str(), color, attr),
                Ok(SearchColumn::Trending) => prep_cell(m.trending_points.to_string().as_str(), color, attr),
                Ok(SearchColumn::Comments) => prep_cell(m.comments.to_string().as_str(), color, attr),
                Ok(SearchColumn::Summary) => prep_cell(m.summary.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::ModidStrs) => prep_cell(m.mod_id_strs.join(",").as_str(), color, attr),
                Ok(SearchColumn::Author) => prep_cell(m.author.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::UrlAliases) => prep_cell(m.url_alias.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::Side) => prep_cell(m.side.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::Type) => prep_cell(m.mod_type.clone().unwrap_or_default().as_str(), color, attr),
                Ok(SearchColumn::Tags) => prep_cell(m.tags.join(",").as_str(), color, attr),
                Ok(SearchColumn::LastReleased) => prep_cell(m.last_released.clone().unwrap_or_default().as_str(), color, attr),
                _ => {Cell::new("")}
            }
        }).collect();

        Row::from(cells)
    }).collect();

    table.set_header(headers);
    table.add_rows(b_rows);

    println!("{}", table);
}

fn prep_cell(text: &str, color: Option<CellColor>, attribute: Option<String>) -> Cell {
    let mut cell = Cell::from(text);

    if color.is_some() {
        info!("Trying to set cell color: {}", color.clone().unwrap());
        cell = cell.fg(Color::from(color.unwrap_or(CellColor::Reset)));
    }

    // TODO: Add actual attribute type so any Comfy_table attribute can be used
    // For now we limit the usable attributes
    if attribute.is_some() {
        let attr: Attribute = match attribute.unwrap().as_str() {
            "bold" => Attribute::Bold,
            "dim" => Attribute::Dim,
            "italic" => Attribute::Italic,
            _ => Attribute::Reset
        };
        cell = cell.add_attribute(attr);
    }

    cell
}