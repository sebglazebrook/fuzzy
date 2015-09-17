extern crate regex;

use regex::Regex;
use std::path::PathBuf;
use std::fs::{self, PathExt};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
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

    // this needs to handle files/directory the user doesn't have permission to read
    pub fn scan(&mut self, current_threads: Arc<AtomicUsize>) {
        match fs::read_dir(&self.root_dir) {
            Ok(read_dir) => {
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() {
                                self.filepaths.push(entry.path().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() {
                                let mut done = false;
                                while !done {
                                    let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                    if current_threads.load(Ordering::Relaxed) < 9 {
                                        current_threads.fetch_add(1, Ordering::Relaxed);
                                        self.threads += 1;
                                        let tx = self.tx.clone();
                                        let spawn_thread_count = current_threads.clone();
                                        thread::spawn(move||{
                                            let mut scanner = DirectoryScanner::new(path);
                                            scanner.scan(spawn_thread_count.clone());
                                            tx.send(scanner);
                                            spawn_thread_count.fetch_sub(1, Ordering::Relaxed);
                                        });
                                        done = true;
                                    } else {
                                        let mut scanner = DirectoryScanner::new(path);
                                        scanner.scan(current_threads.clone());
                                        self.filepaths.extend(scanner.filepaths);
                                        done = true;
                                    }
                                }
                            }
                        }
                        Err(_) => { }
                    }
                }

            }
            Err(_) => { }
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
        scanner.scan(Arc::new(AtomicUsize::new(0)));
        self.result_set.add_many(scanner.filepaths, root_dir.to_str().unwrap());
        self.terminal.show_results(self.result_set.to_vec());
    }

    pub fn apply_filter(&self, regex: Regex) {
        self.terminal.show_results(self.result_set.apply_filter(regex));
    }
}
