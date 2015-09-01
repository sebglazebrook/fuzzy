extern crate regex;

use regex::Regex;
use std::path::Path;
use std::path::PathBuf;
use std::fs::{self, PathExt};
use std::sync::{Arc, Mutex};
use fuzzy::terminal::Terminal;
use fuzzy::result_set::ResultSet;


pub struct FileFinder {
    pub terminal: Arc<Terminal>,
    result_set: ResultSet,
    root_dir: PathBuf
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>, root_dir: PathBuf) -> Arc<Mutex<FileFinder>> {
        Arc::new(Mutex::new(
            FileFinder { 
                terminal: terminal,
                result_set: ResultSet::new(),
                root_dir: root_dir,
            }
        ))
    }

    pub fn start(&mut self) {
        for filepath in self.filepaths_in_directory(&self.root_dir).iter() {
            // TODO paths are absolute, only want relative
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
            match entry {
                Ok(entry) => {
                    filepaths.push(self.clean_file_path(entry.path().into_os_string().into_string().unwrap()));
                    let result = fs::metadata(entry.path());
                    match result {
                        Ok(metadata) => {
                            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                                // each one of these could be a new thread ??
                                let further = self.filepaths_in_directory(&entry.path().as_path());
                                for item in further.iter() {
                                    filepaths.push(self.clean_file_path(item.to_string()).clone());
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        }
        filepaths
    }

    fn clean_file_path(&self, path: String) -> String {
        let cleaned_path = path.replace(self.root_dir.to_str().unwrap(), "");
        if cleaned_path.starts_with("/") {
            cleaned_path[1..].to_string()
        } else {
            cleaned_path
        }
    }
}

