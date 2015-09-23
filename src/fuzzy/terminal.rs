extern crate rustbox;
extern crate time;
extern crate clipboard;

use rustbox::{RustBox, Key, Color};
use self::clipboard::ClipboardContext;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use fuzzy::search_phrase::SearchPhrase;
use std::thread;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::mpsc;

pub struct Terminal {
    pub rustbox: Arc<Mutex<RustBox>>,
    results: Mutex<Vec<String>>,
    hightlighted_result_row: AtomicUsize,
    rx: Arc<Mutex<Receiver<Vec<String>>>>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    search_complete: AtomicBool
}

impl Terminal {

    pub fn new() -> Arc<Terminal> {
        let rustbox = match RustBox::init(Default::default()) {
            Result::Ok(v) => Arc::new(Mutex::new(v)),
            Result::Err(e) => panic!("{}", e),
        };
        let (tx, rx) = mpsc::channel();
        Arc::new(
            Terminal{
                rustbox: rustbox,
                results: Mutex::new(vec![]),
                hightlighted_result_row: AtomicUsize::new(0),
                tx: Arc::new(Mutex::new(tx)),
                rx: Arc::new(Mutex::new(rx)),
                search_complete: AtomicBool::new(false)
            }
        )
    }

    pub fn listen_for_files(&self) {
        let rx = self.rx.clone();
        let (stx, srx) = mpsc::channel();
        thread::spawn(move || {
            let locked_rx = rx.lock().unwrap();
            loop { // need to break out of this
                match locked_rx.try_recv() {
                    Ok(results) => { stx.send(results); }
                    Err(TryRecvError::Disconnected) => { break; }
                    Err(TryRecvError::Empty) => {}
                }
                thread::sleep_ms(1);
            }
        });

        while! self.search_complete.load(Ordering::Relaxed) {
            match srx.try_recv() {
                Ok(results) => { self.show_results(results); }
                Err(TryRecvError::Disconnected) => { break; }
                Err(TryRecvError::Empty) => {}
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
                                ctx.set_contents(self.get_highlighted_result());
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
