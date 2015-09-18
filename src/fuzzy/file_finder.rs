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
                let events;
                {
                    let mut event_service = event_service.lock().unwrap();
                    events = event_service.fetch_all_search_query_change_events();
                }
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
        let _ = self.tx.send(1);
        thread::sleep_ms(1);
    }
}
