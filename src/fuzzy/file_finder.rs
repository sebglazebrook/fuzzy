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
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::mpsc;

pub struct FileFinder {
    pub terminal: Arc<Terminal>,
    event_service: Arc<Mutex<EventService>>,
    result_set: Arc<Mutex<ResultSet>>,
    rx: Arc<Mutex<Receiver<usize>>>,
    tx: Sender<usize>,
    subscriber_channels: Vec<Arc<Mutex<Sender<Vec<String>>>>>,
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
                rx: Arc::new(Mutex::new(rx)),
                subscriber_channels: vec![]
            }
        ))
    }

    pub fn add_subscriber_channel(&mut self, subscriber_channel: Arc<Mutex<Sender<Vec<String>>>>) {
        self.subscriber_channels.push(subscriber_channel);
    }

    pub fn start(&mut self, root_dir: &PathBuf) {
        self.check_for_filters();
        let (tx, rx) = mpsc::channel();
        let mut scanner = DirectoryScanner::new(root_dir.clone(), Arc::new(Mutex::new(tx)));
        thread::spawn(move || {
            scanner.scan(Arc::new(AtomicUsize::new(0)));
        });

        for results in rx.iter() {
            let mut result_set = self.result_set.lock().unwrap();
            result_set.add_many(results, root_dir.to_str().unwrap());
            for subscriber in self.subscriber_channels.iter() {
                let _ = subscriber.lock().unwrap().send(result_set.to_vec());
            }
        }
    }

    fn check_for_filters(&self) {
        let event_service = self.event_service.clone();
        let result_set = self.result_set.clone();
        let rx = self.rx.clone();
        let subscriber_channels = self.subscriber_channels.clone();
        thread::spawn(move|| {
            loop {
                let events;
                {
                    let mut event_service = event_service.lock().unwrap();
                    events = event_service.fetch_all_search_query_change_events();
                }
                if events.len() > 0 {
                    let last_event = events.last().unwrap();
                    let locked_result_set = result_set.lock().unwrap();
                    let filtered_results = locked_result_set.apply_filter(last_event.to_regex());
                    for subscriber in subscriber_channels.iter() {
                        let _ = subscriber.lock().unwrap().send(filtered_results.clone());
                    }
                }
                let locked_rx = rx.lock().unwrap();
                match locked_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }
                thread::sleep_ms(1);
            }
        });
    }

}

impl Drop for FileFinder {

    fn drop(&mut self) {
        self.tx.send(1).unwrap();
        thread::sleep_ms(1); // waiting for the nested thread in FileFinder to be killed.
    }
}
