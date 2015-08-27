extern crate rustbox;
extern crate time;

use rustbox::{RustBox, Key, Color};
use std::error::Error;
use std::sync::{Arc, Mutex};
use fuzzy::search_phrase::SearchPhrase;
use std::thread;
use regex::Regex;

pub struct Terminal {
    pub rustbox: Arc<Mutex<RustBox>>,
    pub results: Vec<String>,
}

impl Terminal {

    pub fn on_stdin<'a>(&self, mut search_phrase: Arc<Mutex<SearchPhrase>>) {
        let mut character_index = 0;
        let mut done = false;
        while !done {
            {
                let rustbox = self.rustbox.clone();
                let rustbox = rustbox.lock().unwrap();
                match rustbox.peek_event(time::Duration::milliseconds(10), false) {
                    Ok(rustbox::Event::KeyEvent(key)) => {
                        match key {
                            Some(Key::Ctrl('c')) => { break; }
                            Some(Key::Char(c)) => { 
                                rustbox.print(character_index, 0, rustbox::RB_NORMAL, Color::White, Color::Black, &c.to_string());
                                rustbox.present();
                                character_index = character_index + 1;

                                // have to do this as a new thread
                                let mut local_search_phrase = search_phrase.clone();
                                thread::spawn(move || {
                                    let mut local_search_phrase = local_search_phrase.lock().unwrap();
                                    local_search_phrase.update(c.to_string());
                                });
                            }
                            Some(Key::Enter) => { done = true; }
                            _ => {  }
                        }
                    },
                    Err(e) => panic!("{}", e.description()),
                    _ => {  }
                }
            }
            thread::sleep_ms(100);
        }
    }

    pub fn show_results(&self, results: Vec<String>) {
        self.clear_results();
        let rustbox = self.rustbox.clone();
        let rustbox = rustbox.lock().unwrap();
        rustbox.print(0, 1, rustbox::RB_NORMAL, Color::White, Color::Black, "Here are the matching results:");
        let mut starting_row = 1;
        for result in results.iter() {
            starting_row = starting_row + 1;
            rustbox.print(0, starting_row, rustbox::RB_NORMAL, Color::White, Color::Black, result);
        }
        rustbox.present();
    }

    pub fn wait_until_exit(&self) {
        let mut done = false;
        while !done {
            let rustbox = self.rustbox.clone();
            let rustbox = rustbox.lock().unwrap();
            println!("Press enter or ctrl+c to exit");
            match rustbox.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    match key {
                        Some(Key::Ctrl('c')) => { break; }
                        Some(Key::Enter) => { done = true; }
                        _ => {  }
                    }
                },
                Err(e) => panic!("{}", e.description()),
                _ => {  }
            }
        }
    }


    fn clear_results(&self) {
        let rustbox = self.rustbox.clone();
        let rustbox = rustbox.lock().unwrap();
        // clear all result rows
        let height = rustbox.height();
        let width = rustbox.width();
        let mut empty_line = String::new();
        for x in 1..width {
            empty_line = empty_line.clone() + " ";
        }
        for x in 1..height {
            rustbox.print(0, x, rustbox::RB_NORMAL, Color::White, Color::Black, &empty_line);
        }
    }
}
