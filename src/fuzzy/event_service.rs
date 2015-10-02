use fuzzy::search_phrase::SearchPhrase;
use fuzzy::terminal::Terminal;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::{Ordering, AtomicBool};

pub struct EventService {
    search_phrases: Vec<SearchPhrase>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    pub rx: Receiver<Vec<String>>,
    pub condvar: Arc<Condvar>
}

impl EventService {

    pub fn new() -> EventService {
        let (tx, rx) = mpsc::channel();
        EventService {
            search_phrases: vec![],
            rx: rx,
            tx: Arc::new(Mutex::new(tx)),
            condvar: Arc::new(Condvar::new())

        }
    }

    pub fn trigger_search_phrase_changed(&mut self, search_phrase: SearchPhrase) {
        self.search_phrases.push(search_phrase);
        self.condvar.notify_all();
    }

    pub fn fetch_all_search_query_change_events(&mut self) -> Vec<SearchPhrase> {
        let search_phrases = self.search_phrases.clone();
        self.search_phrases.clear();
        search_phrases
    }

    pub fn fetch_last_file_finder_event(&mut self) -> Option<Vec<String>> {
        let mut done = false;
        let mut return_value = None;
        while !done {
            let receive_result;
            { receive_result = self.rx.try_recv(); }
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

pub fn listen_for_events(event_service: Arc<Mutex<EventService>>, terminal: Arc<Terminal>, app_finished: Arc<AtomicBool>) {
    thread::spawn(move || {
        while !app_finished.load(Ordering::Relaxed) {
            let receive_result;
            {
                let locked_event_service = event_service.lock().unwrap();
                receive_result = locked_event_service.rx.try_recv();
            }
            match receive_result {
                Ok(result) => { terminal.show_results(result); },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => { break; }
            }
        }
    });
}
