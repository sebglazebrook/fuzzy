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
        let file_finder = FileFinder::new(terminal.clone(), env::current_dir().unwrap());
        App { 
            threads: vec![],
            terminal: terminal,
            file_finder: file_finder,
            rx: rx,
            tx: tx,
        }
    }

    pub fn start(&mut self) -> String {
        self.find_files();
        self.capture_user_input();
        self.wait_until_exit();
        self.get_found_file()
    }

    // --------- private methods ----------- //

    fn find_files(&mut self) {
        let file_finder = self.file_finder.clone();
        let tx = self.tx.clone();
        self.threads.push(true);
        thread::spawn(move|| {
            let mut locked_local_file_finder = file_finder.lock().unwrap();
            locked_local_file_finder.start();
            tx.send(1)
        });
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
}
