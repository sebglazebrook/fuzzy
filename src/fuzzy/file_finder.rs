extern crate regex;

use regex::Regex;
use std::path::PathBuf;
use std::fs::{self, PathExt};
use std::sync::{Arc, Mutex};
use fuzzy::terminal::Terminal;
use fuzzy::result_set::ResultSet;
use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

struct DirectoryScanner {
    root_dir: PathBuf,
    filepaths: Vec<String>,
    threads: usize,
    rx: Receiver<DirectoryScanner>,
    tx: Sender<DirectoryScanner>,
}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf) -> DirectoryScanner {
        let (tx, rx) = mpsc::channel();
        DirectoryScanner{
            root_dir: root_dir,
            filepaths: vec![],
            threads: 0,
            rx: rx,
            tx: tx,
        }
    }

    pub fn scan(&mut self, current_threads: Arc<Mutex<Vec<bool>>>) {
        for entry in fs::read_dir(&self.root_dir).unwrap() {
            match entry {
                Ok(entry) => {
                    let result = fs::metadata(entry.path());
                    match result {
                        Ok(metadata) => {
                            if metadata.is_file() {
                                self.filepaths.push(entry.path().to_str().unwrap().to_string());
                            } else if metadata.is_dir() && !metadata.file_type().is_symlink() {
                                let local_thread_count = current_threads.clone();
                                let mut done = false;
                                while !done {
                                    let mut current_thread_count: usize;
                                    {
                                    let locked_thread_count = local_thread_count.lock().unwrap();
                                    current_thread_count = locked_thread_count.len();
                                    }
                                    let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                    if current_thread_count < 6 {
                                        {
                                            let mut locked_thread_count = local_thread_count.lock().unwrap();
                                            locked_thread_count.push(true);
                                        }
                                        self.threads += 1;
                                        let tx = self.tx.clone();
                                        let spawn_thread_count = current_threads.clone();
                                        thread::spawn(move||{
                                            let mut scanner = DirectoryScanner::new(path);
                                            scanner.scan(spawn_thread_count.clone());
                                            tx.send(scanner);
                                            let mut locked_thread_count = spawn_thread_count.lock().unwrap();
                                            locked_thread_count.pop();   
                                        });
                                        done = true;
                                    } else {
                                        let mut scanner = DirectoryScanner::new(path);
                                        scanner.scan(local_thread_count.clone());
                                        self.filepaths.extend(scanner.filepaths);
                                        done = true;
                                    }
                                }
                            } 
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        }
        for _ in 0..self.threads {
            let scanner = self.rx.recv().unwrap();
            self.filepaths.extend(scanner.filepaths);
        }
    }
}

pub struct FileFinder {
    pub terminal: Arc<Terminal>,
    result_set: ResultSet,
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>) -> Arc<Mutex<FileFinder>> {
        Arc::new(Mutex::new(
            FileFinder { 
                terminal: terminal,
                result_set: ResultSet::new(),
            }
        ))
    }

    pub fn start(&mut self, root_dir: &PathBuf) {
        let mut scanner = DirectoryScanner::new(root_dir.clone());
        scanner.scan(Arc::new(Mutex::new(vec![])));
        self.result_set.add_many(scanner.filepaths, root_dir.to_str().unwrap());
        self.terminal.show_results(self.result_set.to_vec());
    }

    pub fn apply_filter(&self, regex: Regex) {
        self.terminal.show_results(self.result_set.apply_filter(regex));
    }
}
