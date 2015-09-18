extern crate regex;

use regex::Regex;
use std::path::PathBuf;
use std::fs::{self, PathExt};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use fuzzy::terminal::Terminal;
use fuzzy::result_set::ResultSet;
use fuzzy::event_service::EventService;
use std::ops::Drop;
use std::thread;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
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
    event_service: Arc<Mutex<EventService>>,
    result_set: Arc<Mutex<ResultSet>>,
    rx: Arc<Mutex<Receiver<usize>>>,
    tx: Sender<usize>,
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>, event_service: Arc<Mutex<EventService>>) -> Arc<Mutex<FileFinder>> {
        let (tx, rx) = mpsc::channel();
        Arc::new(Mutex::new(
            FileFinder { 
                terminal: terminal,
                event_service: event_service,
                result_set: Arc::new(Mutex::new(ResultSet::new())),
                tx: tx,
                rx: Arc::new(Mutex::new(rx))
            }
        ))
    }

    pub fn start(&mut self, root_dir: &PathBuf) {
        self.check_for_filters();
        let mut scanner = DirectoryScanner::new(root_dir.clone());
        scanner.scan(Arc::new(AtomicUsize::new(0)));
        let mut result_set = self.result_set.lock().unwrap();
        result_set.add_many(scanner.filepaths, root_dir.to_str().unwrap());
        self.terminal.show_results(result_set.to_vec());
    }

    fn check_for_filters(&self) {
        let event_service = self.event_service.clone();
        let terminal = self.terminal.clone();
        let result_set = self.result_set.clone();
        let rx = self.rx.clone();
        thread::spawn(move|| {
            loop {
                let mut event_service = event_service.lock().unwrap();
                let events = event_service.fetch_all_search_query_change_events();
                if events.len() > 0 {
                    let last_event = events.last().unwrap();
                    let locked_result_set = result_set.lock().unwrap();
                    terminal.show_results(locked_result_set.apply_filter(last_event.to_regex()));
                }
                let locked_rx = rx.lock().unwrap();
                match locked_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }
            }
        });
    }

}

impl Drop for FileFinder {

    fn drop(&mut self) {
        self.tx.send(1);
        thread::sleep_ms(1);
    }
}
