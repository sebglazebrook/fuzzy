use fuzzy::search_phrase::SearchPhrase;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

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

pub struct FileFinderEvents {
    data: Vec<Vec<String>>
}

impl FileFinderEvents {

    pub fn new() -> FileFinderEvents {
        FileFinderEvents { data: vec![] }
    }

    pub fn push(&mut self, data: Vec<String>) {
        self.data.push(data);
    }

    pub fn export(&mut self) -> Vec<Vec<String>> {
        let data = self.data.clone();
        self.data.clear();
        data
    }
}

pub struct EventService {
    pub search_phrases: Arc<Mutex<SearchPhrases>>,
    pub file_finder_events: Arc<Mutex<FileFinderEvents>>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    pub rx: Arc<Mutex<Receiver<Vec<String>>>>,
    pub condvar: Arc<Condvar>,
    pub file_finder_condvar: Arc<Condvar>
}

impl EventService {

    pub fn new() -> EventService {
        let (tx, rx) = mpsc::channel();
        EventService {
            search_phrases: Arc::new(Mutex::new(SearchPhrases::new())),
            file_finder_events: Arc::new(Mutex::new(FileFinderEvents::new())),
            rx: Arc::new(Mutex::new(rx)),
            tx: Arc::new(Mutex::new(tx)),
            condvar: Arc::new(Condvar::new()),
            file_finder_condvar: Arc::new(Condvar::new())
        }
    }

    pub fn trigger_search_phrase_changed(&self, search_phrase: SearchPhrase) {
        self.search_phrases.lock().unwrap().push(search_phrase);
        self.condvar.notify_all();
    }

    pub fn trigger_file_finder_event(&self, results: Vec<String>) {
        self.file_finder_events.lock().unwrap().push(results);
        self.file_finder_condvar.notify_all();
    }
}

impl Drop for EventService {

    fn drop(&mut self) {
        self.condvar.notify_all();
    }
}
