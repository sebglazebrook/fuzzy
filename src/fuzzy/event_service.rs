use fuzzy::search_phrase::SearchPhrase;
use fuzzy::terminal::Terminal;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::{Ordering, AtomicBool};

pub struct SearchPhrases {
    data: Vec<SearchPhrase>
}

impl SearchPhrases {

    pub fn new() -> SearchPhrases {
        SearchPhrases { data: vec![] }
    }

    pub fn push(&mut self, data: SearchPhrase) {
        self.data.push(data);
    }

    pub fn export(&mut self) -> Vec<SearchPhrase> {
        let data = self.data.clone();
        self.data.clear();
        data
    }
}

pub struct EventService {
    pub search_phrases: Arc<Mutex<SearchPhrases>>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    pub rx: Arc<Mutex<Receiver<Vec<String>>>>,
    pub condvar: Arc<Condvar>
}

impl EventService {

    pub fn new() -> EventService {
        let (tx, rx) = mpsc::channel();
        EventService {
            search_phrases: Arc::new(Mutex::new(SearchPhrases::new())),
            rx: Arc::new(Mutex::new(rx)),
            tx: Arc::new(Mutex::new(tx)),
            condvar: Arc::new(Condvar::new())
        }
    }

    pub fn trigger_search_phrase_changed(&self, search_phrase: SearchPhrase) {
        self.search_phrases.lock().unwrap().push(search_phrase);
        self.condvar.notify_all();
    }

    pub fn fetch_last_file_finder_event(&self) -> Option<Vec<String>> {
        let mut done = false;
        let mut return_value = None;
        while !done {
            let receive_result;
            {
                let recevier = self.rx.lock().unwrap();
                receive_result = recevier.try_recv();
            }
            match receive_result {
                Ok(result)  => { 
                    return_value = Some(result);
                },
                Err(_) => {
                    done = true;
                }
            } 
        }
        return_value
    }
}

impl Drop for EventService {

    fn drop(&mut self) {
        self.condvar.notify_all();
    }
}

pub fn listen_for_events(event_service: Arc<EventService>, terminal: Arc<Terminal>, app_finished: Arc<AtomicBool>) {
    thread::spawn(move || {
        while !app_finished.load(Ordering::Relaxed) {
            let receive_result;
            {
                receive_result = event_service.rx.lock().unwrap().try_recv();
            }
            match receive_result {
                Ok(result) => { terminal.show_results(result); },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => { break; }
            }
        }
    });
}
