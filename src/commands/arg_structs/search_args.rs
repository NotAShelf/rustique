use clap::{Args, ValueEnum};

#[derive(Args)]
pub struct SearchMods {

    /// This searches by mod text and title
    #[arg(short, long)]
    pub query: Option<String>,
    pub game_version: Option<String>,

    pub game_versions: Option<Vec<String>>,

    #[arg(short, long)]
    pub author: Option<String>,

    #[arg(short, long)]
    pub order_by: Option<OrderBy>,

    #[arg(short = 'O', long)]
    pub order_direction: Option<OrderDirection>,
}



#[derive(ValueEnum, Clone)]
pub enum OrderBy {
    Author,
    AssetId,
    Comments,
    Created,
    Downloads,
    Follows,
    ModId,
    Name,
    Released,
    Trending,
}

#[derive(ValueEnum, Clone)]
pub enum OrderDirection {
    Asc,
    Desc,
}
