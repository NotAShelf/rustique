use std::error::Error;
use std::fmt::format;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{stdin, Read};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use ureq::get;
use zip::ZipArchive;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Row, Table};
use crate::api_structs::ModInfo;
use crate::utils::{extract_all_mods_metadata, extract_zip_metadata, RustiqueOptions};


// TODO:: Should we handle mods that are in directories and not .zip files
pub fn list_installed(rustique_options: RustiqueOptions) -> Result<(), Box<dyn Error>> {
    // TODO: check which platform we are on

    let mods = extract_all_mods_metadata(rustique_options)?;
    let mut table = Table::new();
    // table.set_header(vec!["Name", "ModID", "Version", "Description", "Website"]);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let mut header = Row::new();
    header
        .add_cell(Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Blue))
        .add_cell(Cell::new("ModID").add_attribute(Attribute::Bold).fg(Color::Blue))
        .add_cell(Cell::new("Version").add_attribute(Attribute::Bold).fg(Color::Blue))
        .add_cell(Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Blue))
        .add_cell(Cell::new("Website").add_attribute(Attribute::Bold).fg(Color::Blue));

    table.add_row(header);

    mods.iter().for_each(|mod_info| {
        table.add_row(vec![
            &mod_info.name,
            &mod_info.mod_id,
            &mod_info.version.as_ref().unwrap_or(&"".to_string()),
            &mod_info.description.as_ref().unwrap_or(&"".to_string()),
            &mod_info.website.as_ref().unwrap_or(&"".to_string()),
            ]);
    });

    println!("{}", table);

    Ok(())
}

// pub fn list_installed_mod(mod_id: RustiqueOptions) -> Result<ModInfo, Box<dyn Error>> {
//
//     // list mod info of mod_id from rustique-sync.json, if its available
// }
