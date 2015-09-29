extern crate rustbox;
extern crate regex;
extern crate crossbeam;

use std::env;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;

mod fuzzy;

use fuzzy::search_phrase::SearchPhrase;
use fuzzy::terminal::Terminal;
use fuzzy::file_finder::FileFinder;
use fuzzy::event_service::{self, EventService};
use std::sync::atomic::{Ordering, AtomicBool};

struct App {
    threads: u8,
    terminal: Arc<Terminal>,
    file_finder: Arc<Mutex<FileFinder>>,
    event_service: Arc<Mutex<EventService>>,
    rx: std::sync::mpsc::Receiver<usize>,
    tx: std::sync::mpsc::Sender<usize>,
    app_finished: Arc<AtomicBool>
}

impl App {

    pub fn new() -> App {
        let app_finished = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel();
        let event_service = Arc::new(Mutex::new(EventService::new()));
        let terminal = Terminal::new(event_service.clone());
        event_service::listen_for_events(event_service.clone(), terminal.clone(), app_finished.clone());
        let file_finder = FileFinder::new(terminal.clone(), event_service.clone());
        {
            let tx = event_service.lock().unwrap().tx.clone();
            file_finder.lock().unwrap().add_subscriber_channel(tx);
        }
        App { 
            threads: 0,
            terminal: terminal,
            file_finder: file_finder,
            event_service: event_service,
            rx: rx,
            tx: tx,
            app_finished: app_finished,
        }
    }

    pub fn start(&mut self) -> String {
        self.find_files();
        self.capture_user_input();
        self.prepare_terminal();
        self.wait_until_exit();
        self.get_found_file()
    }

    // --------- private methods ----------- //

    fn prepare_terminal(&self) {
        self.terminal.listen_for_files();
    }

    fn find_files(&mut self) {
        let file_finder = self.file_finder.clone();
        let tx = self.tx.clone();
        self.threads += 1;
        thread::spawn(move|| {
            let mut locked_local_file_finder = file_finder.lock().unwrap();
            locked_local_file_finder.start(&env::current_dir().unwrap());
            tx.send(1)
        });
    }

    fn capture_user_input(&mut self) {
        let search_phrase = Arc::new(Mutex::new(SearchPhrase::init(self.event_service.clone())));
        let tx = self.tx.clone();
        let local_search_phrase = search_phrase.clone();
        let local_terminal = self.terminal.clone();
        self.threads += 1;
        thread::spawn(move || {
            local_terminal.on_stdin(local_search_phrase);
            tx.send(1)
        });

    }

    fn wait_until_exit(&self) {
        self.app_finished.store(true, Ordering::Relaxed);
        for _ in 0..self.threads {
            self.rx.recv().ok().expect("Could not receive answer");
        }
    }

    fn get_found_file(&self) -> String {
        if self.terminal.has_highlighted_result() {
            self.terminal.get_highlighted_result()
        } else {
            String::new()
        }
    }
}

pub fn initialize() {
    let found_file = App::new().start();
    println!("{}", found_file);
    std::process::exit(0);
}
