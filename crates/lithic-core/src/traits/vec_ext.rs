pub trait VecStringExt {
   fn contains_ignore_case(&self, query: &str) -> bool;
   fn contains_any(&self, queries: &[&str]) -> bool;
}

impl VecStringExt for Vec<String> {
   fn contains_ignore_case(&self, query: &str) -> bool {
      let query_lower = query.to_lowercase();
      self.iter().any(|s| s.to_lowercase().contains(&query_lower))
   }

   fn contains_any(&self, queries: &[&str]) -> bool {
      queries.iter().any(|q| self.contains_ignore_case(q))
   }
}
