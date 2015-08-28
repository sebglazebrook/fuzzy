extern crate rustbox;
extern crate regex;

use rustbox::{RustBox};
use std::env;
use std::default::Default;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;

mod fuzzy;
use fuzzy::search_phrase::SearchPhrase;
use fuzzy::terminal::Terminal;
use fuzzy::file_finder::FileFinder;

struct App {
    threads: Vec<bool>,
    terminal: Arc<Terminal>,
    rx: std::sync::mpsc::Receiver<usize>,
    tx: std::sync::mpsc::Sender<usize>,
}

impl App {

    pub fn new() -> App {
        let (tx, rx) = channel();
        App { 
            threads: vec![],
            terminal: Terminal::new(),
            rx: rx,
            tx: tx,
        }
    }

    pub fn start(&mut self) {
        let file_finder = Arc::new(Mutex::new(FileFinder::init(self.terminal.clone())));

        // fetch all the files
        let local_file_finder = file_finder.clone();
        let local_tx = self.tx.clone();
        self.threads.push(true);
        thread::spawn(move|| {
            let path = env::current_dir().unwrap(); // maybe user can send it through as an argument?
            let mut locked_local_file_finder = local_file_finder.lock().unwrap();
            locked_local_file_finder.start(&path);
            local_tx.send(1)
        });
        thread::sleep_ms(50); // wait until some results are found, do this better

        // capture the search phrase
        let search_phrase = Arc::new(Mutex::new(SearchPhrase::init(file_finder.clone())));
        let local_tx = self.tx.clone(); let local_search_phrase = search_phrase.clone();
        let local_terminal = self.terminal.clone();
        self.threads.push(true);
        thread::spawn(move || {
            local_terminal.on_stdin(local_search_phrase);
            local_tx.send(1)
        });

        self.wait_until_exit();
    }

    // --------- private methods ----------- //

    fn wait_until_exit(&self) {
        for _ in self.threads.iter() {
            self.rx.recv().ok().expect("Could not receive answer");
        }
        self.terminal.wait_until_exit();
    }
}

pub fn initialize() {
    App::new().start();
}
