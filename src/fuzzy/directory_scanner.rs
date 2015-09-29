use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs::{self, PathExt};
use std::thread;

pub struct DirectoryScanner {
    root_dir: PathBuf,
    subscriber: Arc<Mutex<Sender<Vec<String>>>>,
    concurrency_limit: usize,
}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf, subscriber: Arc<Mutex<Sender<Vec<String>>>>) -> DirectoryScanner {
        DirectoryScanner{
            root_dir: root_dir,
            threads: 0,
            subscriber: subscriber,
            concurrency_limit: 9,
        }
    }

    pub fn scan(&mut self, current_threads: Arc<AtomicUsize>) {
        match fs::read_dir(&self.root_dir) {
            Ok(read_dir) => {
                let mut filepaths = vec![];
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() {
                                filepaths.push(entry.path().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() {
                                let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                if self.concurrency_limit_reached(&current_threads) {
                                    self.scan_directory(path, current_threads.clone());
                                } else {
                                    self.scan_directory_within_thread(path, current_threads.clone());
                                }
                            }
                        }
                        Err(_) => { }
                    }
                }
                let _ = self.subscriber.lock().unwrap().send(filepaths);
            }
            Err(_) => { }
        }
    }

    //---------- private methods ------------//

    fn concurrency_limit_reached(&self, current_threads: &Arc<AtomicUsize>) -> bool {
        current_threads.load(Ordering::Relaxed) >= self.concurrency_limit
    }

    fn scan_directory(&mut self, path: PathBuf, thread_count: Arc<AtomicUsize>) {
        let subscriber = self.subscriber.clone();
        let mut scanner = DirectoryScanner::new(path, subscriber);
        scanner.scan(thread_count);
    }

    fn scan_directory_within_thread(&mut self, path: PathBuf, thread_count: Arc<AtomicUsize>) {
        thread_count.fetch_add(1, Ordering::Relaxed);
        let spawn_thread_count = thread_count.clone();
        let subscriber = self.subscriber.clone();
        thread::spawn(move||{
            let mut scanner = DirectoryScanner::new(path, subscriber);
            scanner.scan(spawn_thread_count.clone());
            spawn_thread_count.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

