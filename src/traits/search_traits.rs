use std::cmp::Ordering;

pub trait Searchable {
    fn matches_text(&self, query: &str) -> bool;
    fn matches_field(&self, field: &str, value: &str) -> bool;
    fn matches_id(&self, id: u32) -> bool;
    fn matches_tag(&self, tag: &str) -> bool;
}

pub trait Sortable {
    fn get_sort_value(&self, field: &str) -> SortValue;
}

#[derive(Debug, PartialEq)]
pub enum SortValue {
    Number(i64),
    Text(String),
    Date(String),
}

impl PartialOrd for SortValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (SortValue::Number(a), SortValue::Number(b)) => a.partial_cmp(b),
            (SortValue::Text(a), SortValue::Text(b)) => a.partial_cmp(b),
            (SortValue::Date(a), SortValue::Date(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}