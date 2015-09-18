use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs::{self, PathExt};
use std::thread;

pub struct DirectoryScanner {
    pub filepaths: Vec<String>,
    root_dir: PathBuf,
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
                                            let _ = tx.send(scanner);
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

