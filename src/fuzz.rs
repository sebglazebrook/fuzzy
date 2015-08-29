extern crate rustbox;
extern crate regex;

use std::env;
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
    file_finder: Arc<Mutex<FileFinder>>,
    rx: std::sync::mpsc::Receiver<usize>,
    tx: std::sync::mpsc::Sender<usize>,
}

impl App {

    pub fn new() -> App {
        let (tx, rx) = channel();
        let terminal = Terminal::new();
        let file_finder = FileFinder::new(terminal.clone());
        App { 
            threads: vec![],
            terminal: terminal,
            file_finder: file_finder,
            rx: rx,
            tx: tx,
        }
    }

    pub fn start(&mut self) {
        self.find_files();
        self.capture_user_input();
        self.wait_until_exit();
    }

    // --------- private methods ----------- //

    fn find_files(&mut self) {
        let file_finder = self.file_finder.clone();
        let tx = self.tx.clone();
        self.threads.push(true);
        thread::spawn(move|| {
            let path = env::current_dir().unwrap(); // maybe user can send it through as an argument?
            let mut locked_local_file_finder = file_finder.lock().unwrap();
            locked_local_file_finder.start(&path);
            tx.send(1)
        });
        thread::sleep_ms(50); // wait until some results are found, do this better
    }

    fn capture_user_input(&mut self) {
        let search_phrase = Arc::new(Mutex::new(SearchPhrase::init(self.file_finder.clone())));
        let tx = self.tx.clone();
        let local_search_phrase = search_phrase.clone();
        let local_terminal = self.terminal.clone();
        self.threads.push(true);
        thread::spawn(move || {
            local_terminal.on_stdin(local_search_phrase);
            tx.send(1)
        });

    }

    fn wait_until_exit(&self) {
        for _ in self.threads.iter() {
            self.rx.recv().ok().expect("Could not receive answer");
        }
    }
}

pub fn initialize() {
    App::new().start();
}
