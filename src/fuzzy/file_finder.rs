extern crate regex;

use regex::Regex;
use std::path::Path;
use std::fs::{self, PathExt};
use std::sync::{Arc, Mutex};
use fuzzy::terminal::Terminal;
use fuzzy::result_set::ResultSet;


pub struct FileFinder {
    pub terminal: Arc<Terminal>,
    result_set: ResultSet,
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>) -> Arc<Mutex<FileFinder>> {
        Arc::new(Mutex::new(
            FileFinder { 
                terminal: terminal,
                result_set: ResultSet::new()
            }
        ))
    }

    pub fn start(&mut self, dir: &Path) {
        for filepath in self.filepaths_in_directory(&dir).iter() {
            self.result_set.add(filepath.clone())
        };
        self.terminal.show_results(self.result_set.to_vec());
    }

    pub fn apply_filter(&self, regex: Regex) {
        self.terminal.show_results(self.result_set.apply_filter(regex));
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

