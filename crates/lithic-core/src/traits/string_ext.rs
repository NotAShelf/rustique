pub trait StrLowerExt {
    fn lower_contains(&self, needle: &str) -> bool;
    fn contains_str_only(&self, needle: &str) -> bool;
}

impl<T: AsRef<str>> StrLowerExt for T {
    /// checks if needs is in string and ignores case
    fn lower_contains(&self, needle: &str) -> bool {
        let needle_lower = needle.to_lowercase();
        self.as_ref().to_lowercase().contains(&needle_lower)
    }

    /// Checks if needle is in string. Strips all special characters and whitespaces
    fn contains_str_only(&self, needle: &str) -> bool {
        let replace_chars = [
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{', '}', '|',
            '\\', ':', ';', '"', '\'', '<', '>', ',', '.', '?', '/', '~', '`',
        ];

        let needle_lower = needle
            .to_lowercase()
            .split_whitespace()
            .collect::<String>()
            .replace(&replace_chars[..], "");
        let me = self
            .as_ref()
            .to_lowercase()
            .split_whitespace()
            .collect::<String>()
            .replace(&replace_chars[..], "");
        me.contains(&needle_lower)
    }
}
