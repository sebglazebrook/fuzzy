use regex::Regex;

pub struct ResultSet {
    pub results: Vec<String>,
}

impl ResultSet {

    pub fn new() -> ResultSet {
        ResultSet { results: vec![]}
    }

    pub fn add(&mut self, result: String) {
        self.results.push(result);
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.results.clone()
    }

    pub fn apply_filter(&self, regex: Regex) -> Vec<String> {
        let mut matched_results = vec![];
        for content in self.results.iter() {
            if regex.is_match(content) {
                matched_results.push(content.clone());
            }
        }
        matched_results
    }
}
