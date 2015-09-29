extern crate rustbox;
extern crate time;
extern crate clipboard;

use rustbox::{RustBox, Key, Color};
use self::clipboard::ClipboardContext;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use fuzzy::search_phrase::SearchPhrase;
use fuzzy::event_service::EventService;
use std::thread;
use std::sync::mpsc::{Sender};
use std::sync::mpsc;

pub struct Terminal {
    pub rustbox: Arc<Mutex<RustBox>>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    event_service: Arc<Mutex<EventService>>,
    results: Mutex<Vec<String>>,
    hightlighted_result_row: AtomicUsize,
    search_complete: AtomicBool,
    number_of_results: AtomicUsize,
}

impl Terminal {

    pub fn new(event_service: Arc<Mutex<EventService>>) -> Arc<Terminal> {
        let rustbox = match RustBox::init(Default::default()) {
            Result::Ok(v) => Arc::new(Mutex::new(v)),
            Result::Err(e) => panic!("{}", e),
        };
        let (tx, _) = mpsc::channel();
        Arc::new(
            Terminal{
                rustbox: rustbox,
                event_service: event_service,
                results: Mutex::new(vec![]),
                hightlighted_result_row: AtomicUsize::new(0),
                tx: Arc::new(Mutex::new(tx)),
                search_complete: AtomicBool::new(false),
                number_of_results: AtomicUsize::new(0)
            }
        )
    }

    pub fn listen_for_files(&self) {
        while !self.search_complete.load(Ordering::Relaxed) {
            let event_option;
            {
                let mut locked_event_service = self.event_service.lock().unwrap();
                event_option = locked_event_service.fetch_last_file_finder_event();
            }
            match event_option {
                Some(result) => {
                    self.show_results(result);
                },
                None => {}
            }
        }
    }

    pub fn on_stdin(&self, search_phrase: Arc<Mutex<SearchPhrase>>) {
        let mut character_index = 0;
        let mut done = false;
        while !done {
            {
                let rustbox = self.rustbox.clone();
                let rustbox = rustbox.lock().unwrap();
                match rustbox.peek_event(time::Duration::microseconds(1), false) {
                    Ok(rustbox::Event::KeyEvent(key)) => {
                        match key {
                            Some(Key::Ctrl('c')) => { break; }
                            Some(Key::Char(c)) => { 
                                rustbox.print(character_index, 0, rustbox::RB_REVERSE, Color::White, Color::Black, &c.to_string());
                                rustbox.present();
                                character_index = character_index + 1;

                                // have to do this as a new thread but don't want to 
                                let local_search_phrase = search_phrase.clone();
                                // do we have to check to make sure this thread is killed properly
                                thread::spawn(move || {
                                    let mut local_search_phrase = local_search_phrase.lock().unwrap();
                                    local_search_phrase.update(c.to_string()); 
                                });
                            }
                            Some(Key::Backspace) => {
                                let index: usize;
                                if character_index != 0 {
                                    index =  character_index - 1;
                                } else {
                                    index =  character_index;
                                }
                                rustbox.print(index, 0, rustbox::RB_NORMAL, Color::White, Color::Black, " ");
                                rustbox.present();
                                if character_index != 0 {
                                    character_index = character_index - 1;
                                }

                                // have to do this as a new thread
                                let local_search_phrase = search_phrase.clone();
                                // do we have to make sure this thread is killed properly?
                                thread::spawn(move || {
                                    let mut local_search_phrase = local_search_phrase.lock().unwrap();
                                    local_search_phrase.delete_last();
                                });
                            }
                            Some(Key::Ctrl('j')) => {
                                self.hightlight_next_row(&rustbox);
                            }
                            Some(Key::Ctrl('k')) => {
                                self.hightlight_previous_row(&rustbox);
                            }
                            Some(Key::Ctrl('y')) => {
                                let mut ctx = ClipboardContext::new().unwrap();
                                let _ = ctx.set_contents(self.get_highlighted_result());
                                done = true;
                            }
                            Some(Key::Down) => {
                                self.hightlight_next_row(&rustbox);
                            }
                            Some(Key::Up) => {
                                self.hightlight_previous_row(&rustbox);
                            }
                            Some(Key::Enter) => { done = true; }
                            _ => {  }
                        }
                    },
                    Err(e) => panic!("{}", e.description()),
                    _ => {  }
                }
            }
        }
        self.search_complete.store(true, Ordering::Relaxed);
    }

    pub fn show_results(&self, results: Vec<String>) {
        self.clear_results();
        let rustbox = self.rustbox.clone();
        let rustbox = rustbox.lock().unwrap();
        let max_displayed_results;
        if results.len() > rustbox.height() {
            max_displayed_results = rustbox.height();
        } else {
            max_displayed_results = results.len();
        }
        // clean old status bar
        let mut empty_string = String::new();
        for _ in 1..self.number_of_results.load(Ordering::Relaxed).to_string().len() {
            empty_string = empty_string.clone() + " ";
        }
        let x_value = rustbox.width() - self.number_of_results.load(Ordering::Relaxed).to_string().len();
        rustbox.print(x_value, 0, rustbox::RB_NORMAL, Color::White, Color::Black, &empty_string);

        // new status bar
        let x_value = rustbox.width() - results.len().to_string().len();
        rustbox.print(x_value, 0, rustbox::RB_NORMAL, Color::White, Color::Black, &results.len().to_string());
        self.number_of_results.store(results.len(), Ordering::Relaxed);

        for index in 0..max_displayed_results {
            rustbox.print(0, index + 1, rustbox::RB_NORMAL, Color::White, Color::Black, &results[index]);
        }
        rustbox.present();
        let mut locked_results = self.results.lock().unwrap();
        locked_results.clear();
        locked_results.extend(results);
    }

    fn clear_results(&self) {
        let rustbox = self.rustbox.clone();
        let rustbox = rustbox.lock().unwrap();
        // clear all result rows
        let mut empty_line = String::new(); // TODO there must be a better way of doing this in rust
        for _ in 1..(rustbox.width() + 1) {
            empty_line = empty_line.clone() + " ";
        }
        for row in 1..rustbox.height() {
            rustbox.print(0, row, rustbox::RB_NORMAL, Color::White, Color::Black, &empty_line);
        }
    }

    fn hightlight_next_row(&self, rustbox: &RustBox) {
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if self.hightlighted_result_row.load(Ordering::Relaxed) > 0 {
            rustbox.print(0, self.hightlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::White, Color::Black, &results[(self.hightlighted_result_row.load(Ordering::Relaxed) - 1)]);
        }
        // highlight next row
        self.hightlighted_result_row.fetch_add(1, Ordering::Relaxed);
        rustbox.print(0, self.hightlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(self.hightlighted_result_row.load(Ordering::Relaxed) - 1)]);
        rustbox.present();
    }

    fn hightlight_previous_row(&self, rustbox: &RustBox) {
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if self.hightlighted_result_row.load(Ordering::Relaxed) > 0 {
            rustbox.print(0, self.hightlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::White, Color::Black, &results[(self.hightlighted_result_row.load(Ordering::Relaxed) - 1)]);
            if self.hightlighted_result_row.load(Ordering::Relaxed) > 1 {
                // hightlight the previous row
                self.hightlighted_result_row.fetch_sub(1, Ordering::Relaxed);
                rustbox.print(0, self.hightlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(self.hightlighted_result_row.load(Ordering::Relaxed) - 1)]);
            } else {
                self.hightlighted_result_row.store(0, Ordering::Relaxed)
            }
            rustbox.present();
        }
    }

    pub fn has_highlighted_result(&self) -> bool {
        self.hightlighted_result_row.load(Ordering::Relaxed) > 0
    }

    pub fn get_highlighted_result(&self) -> String {
        let index = self.hightlighted_result_row.load(Ordering::Relaxed);
        index.to_string();
        self.results.lock().unwrap()[index - 1].clone()
    }
}
