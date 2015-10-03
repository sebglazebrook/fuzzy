extern crate regex;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use fuzzy::terminal::Terminal;
use fuzzy::result_set::ResultSet;
use fuzzy::event_service::EventService;
use fuzzy::directory_scanner::DirectoryScanner;
use std::ops::Drop;
use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

pub struct FileFinder {
    pub terminal: Arc<Terminal>,
    event_service: Arc<EventService>,
    result_set: Arc<Mutex<ResultSet>>,
    tx: Sender<usize>,
    subscriber_channels: Vec<Arc<Mutex<Sender<Vec<String>>>>>,
    root_dir: PathBuf
}

impl FileFinder {

    pub fn new(terminal: Arc<Terminal>, event_service: Arc<EventService>) -> Arc<Mutex<FileFinder>> {
        let (tx, _) = mpsc::channel();
        Arc::new(Mutex::new(
            FileFinder { 
                terminal: terminal,
                event_service: event_service,
                result_set: Arc::new(Mutex::new(ResultSet::new())),
                tx: tx,
                subscriber_channels: vec![],
                root_dir: PathBuf::new()
            }
        ))
    }

    pub fn add_subscriber_channel(&mut self, subscriber_channel: Arc<Mutex<Sender<Vec<String>>>>) {
        self.subscriber_channels.push(subscriber_channel);
    }

    pub fn start(&mut self, root_dir: &PathBuf) {
        self.root_dir = root_dir.clone();
        self.listen_for_filters();
        let (tx, rx) = mpsc::channel();
        let mut scanner = DirectoryScanner::new(root_dir.clone(), Arc::new(Mutex::new(tx)));
        thread::spawn(move || {
            scanner.scan(Arc::new(AtomicUsize::new(0)));
            // what checks this thread and make sure it's killed properly
        });
        self.listen_for_scanner_updates(rx);
        self.update_subscribers();
    }

    // ----------- private methods ---------- //

    fn listen_for_scanner_updates(&self, receiver: Receiver<Vec<String>>) {
        for results in receiver.iter() {
            let mut result_set = self.result_set.lock().unwrap();
            result_set.add_many(results, self.root_dir.to_str().unwrap());
            if result_set.number_of_results() < 100 {
                for subscriber in self.subscriber_channels.iter() {
                    let _ = subscriber.lock().unwrap().send(result_set.to_vec());
                }
            } else {
                // update count here?
            }
        }
    }

    fn update_subscribers(&self) {
        let result_set = self.result_set.lock().unwrap();
        for subscriber in self.subscriber_channels.iter() {
            let _ = subscriber.lock().unwrap().send(result_set.to_vec());
        }
    }

    fn listen_for_filters(&self) {
        let event_service = self.event_service.clone();
        let result_set = self.result_set.clone();
        let subscriber_channels = self.subscriber_channels.clone();
        thread::spawn(move|| {
            loop {
                let condvar = event_service.condvar.clone();
                let mut search_phrases = event_service.search_phrases.lock().unwrap();
                search_phrases = condvar.wait(search_phrases).unwrap();
                let events = search_phrases.export();
                if events.len() > 0 {
                    let last_event = events.last().unwrap();
                    let locked_result_set = result_set.lock().unwrap();
                    let filtered_results = locked_result_set.apply_filter(last_event.to_regex());
                    for subscriber in subscriber_channels.iter() {
                        let _ = subscriber.lock().unwrap().send(filtered_results.clone());
                    }
                } else {
                    break;
                }
            }
        });
    }
}

impl Drop for FileFinder {

    fn drop(&mut self) {
        let _ = self.tx.send(1);
        thread::sleep_ms(1); // waiting for the nested thread in FileFinder to be killed.
    }
}
