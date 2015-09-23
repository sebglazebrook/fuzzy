use regex::Regex;

pub struct ResultSet {
    pub results: Vec<String>,
}

impl ResultSet {

    pub fn new() -> ResultSet {
        ResultSet { results: vec![]}
    }

    pub fn add_many(&mut self, results: Vec<String>, root_dir: &str) {
        let mut new = vec![];
        for result in results {
            new.push(result.replace(root_dir, "")[1..].to_string());
        }
        self.results.extend(new);
        self.sort_results();
        self.results.dedup();
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

    //-------- private methods -------------//

    fn sort_results(&mut self) {
        self.results.sort();
    }
}

impl Clone for ResultSet {
    
    fn clone(&self) -> ResultSet {
        ResultSet { results: self.to_vec() }
    }
}
