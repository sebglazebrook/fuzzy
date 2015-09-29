use crossbeam;
use regex::Regex;
use std::sync::mpsc::channel;

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
            let mut sanitized_string = result.clone();
            if root_dir != "/" {
                sanitized_string = result.replace(root_dir, "")[1..].to_string();
            }
            new.push(sanitized_string);
        }
        self.results.extend(new);
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.results.clone()
    }

    pub fn apply_filter(&self, regex: Regex) -> Vec<String> {
        let mut matched_results = vec![];
        let mut receivers = vec![];
        crossbeam::scope(|scope| {
            let filter_concurrency_limit = 8;
            let chunk_length = self.results.len() / filter_concurrency_limit;
            for chunk in self.results.chunks(chunk_length) {
                let (tx, rx) = channel();
                receivers.push(rx);
                let local_regex = regex.clone();
                scope.spawn(move || {
                    let mut local_matches = vec![];
                    for content in chunk.iter() {
                        if local_regex.is_match(content) {
                            local_matches.push(content.clone());
                        }
                    }
                    let _ = tx.send(local_matches);
                });
            }
        });

        for receiver in receivers.iter() {
            let local_matches = receiver.recv().unwrap();
            matched_results.extend(local_matches);
        }
        matched_results
    }


    pub fn number_of_results(&self) -> usize {
        self.results.len()
    }
}

impl Clone for ResultSet {
    
    fn clone(&self) -> ResultSet {
        ResultSet { results: self.to_vec() }
    }
}
