extern crate regex;
extern crate rustbox;

use std::env;
use rustbox::{RustBox};
use std::default::Default;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;

mod fuzzy;
use fuzzy::search_phrase::SearchPhrase;
use fuzzy::terminal::Terminal;
use fuzzy::file_finder::FileFinder;

struct App;

impl App {

    pub fn new() -> App {
        App
    }

    pub fn start(&self) {
        let mut threads = vec![];
        let (tx, rx) = channel();
        let rustbox = match RustBox::init(Default::default()) {
            Result::Ok(v) => Arc::new(Mutex::new(v)),
            Result::Err(e) => panic!("{}", e),
        };
        let terminal = Arc::new(Terminal {rustbox: rustbox, results: vec![] } );
        let file_finder = Arc::new(Mutex::new(FileFinder::init(terminal.clone())));

        // fetch all the files
        let local_tx = tx.clone();
        let local_file_finder = file_finder.clone();
        threads.push(true);
        thread::spawn(move|| {
            let path = env::current_dir().unwrap(); // maybe user can send it through as an argument?
            let mut locked_local_file_finder = local_file_finder.lock().unwrap();
            locked_local_file_finder.start(&path);
            local_tx.send(1)
        });
        thread::sleep_ms(50); // wait until some results are found, do this better

        // capture the search phrase
        let search_phrase = Arc::new(Mutex::new(SearchPhrase::init(file_finder.clone())));
        let local_tx = tx.clone(); let local_search_phrase = search_phrase.clone();
        let local_terminal = terminal.clone();
        threads.push(true);
        thread::spawn(move || {
            local_terminal.on_stdin(local_search_phrase);
            local_tx.send(1)
        });

        for _ in threads.iter() {
            rx.recv().ok().expect("Could not receive answer");
        }

        // wait for everything to finish
        terminal.wait_until_exit();

    }
}

pub fn initialize() {
    App::new().start();
}
