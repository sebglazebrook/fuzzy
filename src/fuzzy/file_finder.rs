extern crate regex;

use regex::Regex;
use std::path::Path;
use std::fs::{self, PathExt};
use std::sync::Arc;
use fuzzy::terminal::Terminal;

pub struct FileFinder {
    pub results: Vec<String>,
    pub terminal: Arc<Terminal>,
}

impl FileFinder {

    pub fn init(terminal: Arc<Terminal>) -> FileFinder {
        FileFinder { 
            results: vec![],
            terminal: terminal
        } 
    }

    pub fn start(&mut self, dir: &Path) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            self.results.push(entry.path().into_os_string().into_string().unwrap());
            let attr = fs::metadata(entry.path()).unwrap();
            if attr.is_dir() {
                // each one of these could be a new thread ??
                let further = get_directory_contents(&entry.path().as_path());
                for item in further.iter() {
                    self.results.push(item.to_string());
                }
            } 
            self.terminal.show_results(self.results.clone());
        }
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
}

fn get_directory_contents(dir: &Path) -> Vec<String> {
    let mut results: Vec<String> = vec![];
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        results.push(entry.path().into_os_string().into_string().unwrap());
        let attr = fs::metadata(entry.path()).unwrap();
        if attr.is_dir() {
            // each one of these could be a new thread ??
            let further = get_directory_contents(&entry.path().as_path());
            for item in further.iter() {
                results.push(item.to_string());
            }
        } 
    }
    results
}
