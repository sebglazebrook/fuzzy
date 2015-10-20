use crossbeam;
use regex::Regex;
use std::sync::mpsc::channel;

pub struct ResultSet {
    results: Vec<String>,
    filtered_results: Vec<String>,
    filter_string: String,
    filter_applied: bool
}

impl ResultSet {

    pub fn new() -> ResultSet {
        ResultSet { results: vec![], filter_string: String::from("*"), filter_applied: false, filtered_results: vec![] }
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
        if self.filter_applied {
            self.re_run_filter()
        }
    }

    pub fn to_vec(&self) -> Vec<String> {
        if self.filter_applied {
            self.filtered_results.clone()
        } else {
            self.results.clone()
        }
    }

    pub fn apply_filter(&mut self, regex: Regex) -> Vec<String> {
        if self.filter_applied && self.additive_filter(&regex) {
            self.apply_to_filtered(regex)
        } else {
            self.apply_to_all(regex)
        }
    }

    pub fn number_of_results(&self) -> usize {
        self.results.len()
    }

    // ------ private methods ----------//

    fn re_run_filter(&mut self) {
        let string = self.filter_string.clone();
        self.apply_filter(Regex::new(&string).unwrap());
    }

    fn additive_filter(&self, regex: &Regex) -> bool {
        String::from(regex.as_str()).contains(&self.filter_string)
    }

    fn apply_to_filtered(&mut self, regex: Regex) -> Vec<String> {
        self.filtered_results  = filter_collection(&mut self.filtered_results, &regex);
        self.filter_string = regex.to_string();
        self.filter_applied = true;
        self.filtered_results.clone()
    }

    fn apply_to_all(&mut self, regex: Regex) -> Vec<String> {
        self.filtered_results = filter_collection(&mut self.results, &regex);
        self.filter_string = regex.to_string();
        self.filter_applied = true;
        self.filtered_results.clone()
    }
}

impl Clone for ResultSet {

    fn clone(&self) -> ResultSet {
        ResultSet { results: self.to_vec(), filter_string: String::from("*"), filter_applied: self.filter_applied, filtered_results: self.filtered_results.clone() }
    }
}

fn filter_collection(collection: &Vec<String>, regex: &Regex) -> Vec<String> {
        let mut matched_results = vec![];
        let mut receivers = vec![];

        crossbeam::scope(|scope| {
            let filter_concurrency_limit = 8;
            let chunk_length = collection.len() / filter_concurrency_limit;
            for chunk in collection.chunks(chunk_length) {
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
