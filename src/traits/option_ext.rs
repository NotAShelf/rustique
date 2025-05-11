use crate::traits::string_ext::StrLowerExt;

pub trait OptionExt {
    type Inner;

    fn matches_contains(&self, query: &str) -> bool;
    fn as_str_option(&self) -> Option<&str>;
    fn as_u32_option(&self) -> Option<u32>;
}

impl OptionExt for Option<String> {
    type Inner = String;
    fn matches_contains(&self, query: &str) -> bool {
        self.as_ref()
            .map(|s| s.lower_contains(&query.to_lowercase()))
            .unwrap_or(false)
    }

    fn as_str_option(&self) -> Option<&str> {
        self.as_deref()
    }

    fn as_u32_option(&self) -> Option<u32> {
        None
    }
}