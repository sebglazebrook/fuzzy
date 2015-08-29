extern crate regex;

use regex::Regex;
use std::path::Path;
use std::fs::{self, PathExt};
use fuzzy::terminal::Terminal;
use std::sync::{Arc, Mutex};

pub struct FileFinder {
    pub results: Vec<String>,
    pub terminal: Arc<Terminal>,
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>) -> Arc<Mutex<FileFinder>> {
        Arc::new(Mutex::new(FileFinder { results: vec![], terminal: terminal }))
    }

    pub fn start(&mut self, dir: &Path) {
        for filepath in self.filepaths_in_directory(&dir).iter() {
            self.results.push(filepath.clone());
        };
        self.terminal.show_results(self.results.clone());
    }

    pub fn apply_filter(&self, regex: Regex) {
        let mut matched_results = vec![];
        for content in self.results.iter() {
            if regex.is_match(content) {
                matched_results.push(content.clone());
            }
        }
        self.terminal.show_results(matched_results.clone());
    }

    // --------- private methods --------

    fn filepaths_in_directory(&self, dir: &Path) -> Vec<String> {
        let mut filepaths = vec![];
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            filepaths.push(self.sanitize_file_path(entry.path().into_os_string().into_string().unwrap()));
            let attr = fs::metadata(entry.path()).unwrap();
            if attr.is_dir() {
                // each one of these could be a new thread ??
                let further = self.filepaths_in_directory(&entry.path().as_path());
                for item in further.iter() {
                    filepaths.push(self.sanitize_file_path(item.to_string()).clone());
                }
            }
        }
        filepaths
    }

    fn sanitize_file_path(&self, path: String) -> String {
        path[1..].to_string()
    }
}

