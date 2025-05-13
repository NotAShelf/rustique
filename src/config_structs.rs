use std::fmt::Display;
use std::str::FromStr;
use clap::ValueEnum;
use comfy_table::{Color};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;
use crate::flatten_map::FlattenMap;

#[derive(Deserialize, Debug)]
pub struct Tables {
    pub list: TableSection,
    pub search: TableSection
}

impl Serialize for Tables {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("list", &self.list)?;
        map.serialize_entry("search", &self.search)?;
        map.end()
    }
}

impl Tables {
   pub fn with_defaults() -> Self {
         let mut list = TableSection::new();

        // List headers
        list.headers.with(ListColumn::Name.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::ModId.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::Version.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::LatestVersion.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::Deps.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::MissingDeps.as_str(), Some(CellColor::Green), Some("bold"))
                    .with(ListColumn::Description.as_str(), Some(CellColor::Green), Some("bold"));

        // List cells
        list.cells.with(ListColumn::Name.as_str(), Some(CellColor::Yellow), None)
                  .with(ListColumn::ModId.as_str(), Some(CellColor::Reset), None)
                  .with(ListColumn::Version.as_str(), Some(CellColor::Reset), Some("dim"))
                  .with(ListColumn::LatestVersion.as_str(), Some(CellColor::Green), None)
                  .with(ListColumn::Deps.as_str(), Some(CellColor::Reset), None)
                  .with(ListColumn::MissingDeps.as_str(), Some(CellColor::Red), Some("bold"))
                  .with(ListColumn::Description.as_str(), Some(CellColor::Reset), None);

        let mut search = TableSection::new();

        // Search headers
        search.headers.with("mod_id", Some(CellColor::Green), Some("bold"))
                      .with("name", Some(CellColor::Green), Some("bold"))
                      .with("summary", Some(CellColor::Green), Some("bold"));

        // Search cells
        search.cells.with("mod_id", Some(CellColor::Magenta), Some("bold"))
                    .with("name", Some(CellColor::Reset), None)
                    .with("summary", Some(CellColor::Reset), None);

        Self {
            list,
            search,
        }
   }
}

#[derive(Deserialize, Debug)]
pub struct TableSection
{
    pub headers: FlattenMap,
    pub cells: FlattenMap,
}

impl Serialize for TableSection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("headers", &self.headers)?;
        map.serialize_entry("cells", &self.cells)?;
        map.end()
    }
}

impl Default for TableSection {
    fn default() -> Self {
        Self {
            headers: FlattenMap::new(),
            cells: FlattenMap::new(),
        }
    }
}

impl TableSection {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ColumnProperties {
    pub color: Option<CellColor>,
    pub attribute: Option<String>
}

impl Default for ColumnProperties {
    fn default() -> Self {
        Self {
            color: Option::from(CellColor::Reset),
            attribute: Option::from(String::new())
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ModPack {
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AliasConfig {
    pub name: String,
    pub mod_dir: String,
    pub pinned_game_version: String,
}


#[derive(ValueEnum, Deserialize,Debug, Clone)]
pub enum CellColor {
    Black,
    Blue,
    Cyan,
    DarkCyan,
    DarkBlue,
    Green,
    DarkGreen,
    Grey,
    DarkGrey,
    Magenta,
    DarkMagenta,
    Red,
    DarkRed,
    White,
    Yellow,
    DarkYellow,
    Reset,
}

impl Serialize for CellColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self {
            Self::Black => serializer.serialize_str("black"),
            Self::Blue => serializer.serialize_str("blue"),
            Self::DarkBlue => serializer.serialize_str("dark_blue"),
            Self::Cyan => serializer.serialize_str("cyan"),
            Self::DarkCyan => serializer.serialize_str("dark_cyan"),
            Self::Green => serializer.serialize_str("green"),
            Self::DarkGreen => serializer.serialize_str("dark_green"),
            Self::Grey => serializer.serialize_str("grey"),
            Self::DarkGrey => serializer.serialize_str("dark_grey"),
            Self::Magenta => serializer.serialize_str("magenta"),
            Self::DarkMagenta => serializer.serialize_str("dark_magenta"),
            Self::Red => serializer.serialize_str("red"),
            Self::DarkRed => serializer.serialize_str("dark_red"),
            Self::White => serializer.serialize_str("white"),
            Self::Yellow => serializer.serialize_str("yellow"),
            Self::DarkYellow => serializer.serialize_str("dark_yellow"),
            Self::Reset => serializer.serialize_str("reset"),
        }
    }
}


// This makes it easier to directly map our enum to Color so it can be used with ValueEnum for CLI
impl From<CellColor> for Color {
    fn from(value: CellColor) -> Self {
        match value {
            CellColor::Black        => Color::Black,
            CellColor::Blue         => Color::Blue,
            CellColor::DarkBlue     => Color::DarkBlue,
            CellColor::Green        => Color::Green,
            CellColor::DarkGreen    => Color::DarkGreen,
            CellColor::Grey         => Color::Grey,
            CellColor::DarkGrey     => Color::DarkGrey,
            CellColor::Magenta      => Color::Magenta,
            CellColor::DarkMagenta  => Color::DarkMagenta,
            CellColor::Red          => Color::Red,
            CellColor::DarkRed      => Color::DarkRed,
            CellColor::White        => Color::White,
            CellColor::Yellow       => Color::Yellow,
            CellColor::DarkYellow   => Color::DarkYellow,
            CellColor::Reset        => Color::Reset,
            _ => Color::Reset,
        }
    }
}

impl Display for CellColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellColor::Black => write!(f, "black"),
            CellColor::Blue => write!(f, "blue"),
            CellColor::DarkBlue => write!(f, "dark_blue"),
            CellColor::Cyan => write!(f, "cyan"),
            CellColor::DarkCyan => write!(f, "dark_cyan"),
            CellColor::Green => write!(f, "green"),
            CellColor::DarkGreen => write!(f, "dark_green"),
            CellColor::Grey => write!(f, "grey"),
            CellColor::DarkGrey => write!(f, "dark_grey"),
            CellColor::Magenta => write!(f, "magenta"),
            CellColor::DarkMagenta => write!(f, "dark_magenta"),
            CellColor::Red => write!(f, "red"),
            CellColor::DarkRed => write!(f, "dark_red"),
            CellColor::Reset => write!(f, "reset"),
            CellColor::White => write!(f, "white"),
            CellColor::Yellow => write!(f, "yellow"),
            CellColor::DarkYellow => write!(f, "dark_yellow"),
            _=> Err(std::fmt::Error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListColumn {
    Name,
    ModId,
    Version,
    LatestVersion,
    Deps,
    MissingDeps,
    Changelog,
    Description,
    Website,
    GameVersion,
    LastUpdateLocal,
    LastUpdateRemote,
    PinnedVersion,
    HasBackup,
    Filename,
}

impl ListColumn {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::ModId => "mod_id",
            Self::Version => "version",
            Self::LatestVersion => "latest_version",
            Self::GameVersion => "game_version",
            Self::PinnedVersion => "pinned_version",
            Self::Deps => "deps",
            Self::MissingDeps => "missing_deps",
            Self::Changelog => "changelog",
            Self::Description => "description",
            Self::Website => "website",
            Self::LastUpdateLocal => "last_update",
            Self::LastUpdateRemote => "last_update_remote",
            Self::HasBackup => "has_backup",
            Self::Filename => "filename",
        }
    }
}

impl FromStr for ListColumn {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "mod_id" => Ok(Self::ModId),
            "version" => Ok(Self::Version),
            "latest_version" => Ok(Self::LatestVersion),
            "deps" => Ok(Self::Deps),
            "missing_deps" => Ok(Self::MissingDeps),
            "changelog" => Ok(Self::Changelog),
            "description" => Ok(Self::Description),
            "website" => Ok(Self::Website),
            "game_version" => Ok(Self::GameVersion),
            "last_update_local" => Ok(Self::LastUpdateLocal),
            "last_update_remote" => Ok(Self::LastUpdateRemote),
            "pinned_version" => Ok(Self::PinnedVersion),
            "has_backup" => Ok(Self::HasBackup),
            "filename" => Ok(Self::Filename),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchColumn {
    Name,
    ModId,
    Downloads,
    Follows,
    Trending,
    Comments,
    Summary,
    ModidStrs,
    AssetId,
    Author,
    UrlAliases,
    Side,
    Type,
    Tags,
    LastReleased,
}

impl SearchColumn {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::ModId => "mod_id",
            Self::AssetId => "asset_id",
            Self::Downloads => "downloads",
            Self::Follows => "follows",
            Self::Trending => "trending",
            Self::Comments => "comments",
            Self::Summary => "summary",
            Self::ModidStrs => "modid_strs",
            Self::Author => "author",
            Self::UrlAliases => "url_aliases",
            Self::Side => "side",
            Self::Type => "type",
            Self::Tags => "tags",
            Self::LastReleased => "last_released",
        }
    }
}


impl FromStr for SearchColumn {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "mod_id" => Ok(Self::ModId),
            "asset_id" => Ok(Self::AssetId),
            "downloads" => Ok(Self::Downloads),
            "follows" => Ok(Self::Follows),
            "trending" => Ok(Self::Trending),
            "comments" => Ok(Self::Comments),
            "summary" => Ok(Self::Summary),
            "modid_strs" => Ok(Self::ModidStrs),
            "author" => Ok(Self::Author),
            "url_aliases" => Ok(Self::UrlAliases),
            "side" => Ok(Self::Side),
            "type" => Ok(Self::Type),
            "tags" => Ok(Self::Tags),
            "last_released" => Ok(Self::LastReleased),
            _ => Err(())
        }
    }
}