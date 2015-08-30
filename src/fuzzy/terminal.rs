extern crate rustbox;
extern crate time;

use rustbox::{RustBox, Key, Color};
use std::error::Error;
use std::sync::{Arc, Mutex};
use fuzzy::search_phrase::SearchPhrase;
use std::thread;

pub struct Terminal {
    pub rustbox: Arc<Mutex<RustBox>>,
    results: Mutex<Vec<String>>,
    hightlighted_result_row: Mutex<Vec<usize>>,
}

impl Terminal {

    pub fn new() -> Arc<Terminal> {
        let rustbox = match RustBox::init(Default::default()) {
            Result::Ok(v) => Arc::new(Mutex::new(v)),
            Result::Err(e) => panic!("{}", e),
        };
        Arc::new(Terminal {rustbox: rustbox, results: Mutex::new(vec![]), hightlighted_result_row: Mutex::new(vec![0]) } )
    }

    pub fn on_stdin<'a>(&self, search_phrase: Arc<Mutex<SearchPhrase>>) {
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

                                // have to do this as a new thread
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
            thread::sleep_ms(1);
        }
    }

    pub fn show_results(&self, results: Vec<String>) {
        self.clear_results();
        let rustbox = self.rustbox.clone();
        let rustbox = rustbox.lock().unwrap();
        let mut starting_row = 0;
        for result in results.iter() {
            starting_row = starting_row + 1;
            rustbox.print(0, starting_row, rustbox::RB_NORMAL, Color::White, Color::Black, result);
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
        for _ in 1..rustbox.width() {
            empty_line = empty_line.clone() + " ";
        }
        for row in 1..rustbox.height() {
            rustbox.print(0, row, rustbox::RB_NORMAL, Color::White, Color::Black, &empty_line);
        }
    }

    fn hightlight_next_row(&self, rustbox: &RustBox) {
        let mut hightlighted_result_row = self.hightlighted_result_row.lock().unwrap();
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if hightlighted_result_row[0] > 0 {
            rustbox.print(0, hightlighted_result_row[0], rustbox::RB_NORMAL, Color::White, Color::Black, &results[(hightlighted_result_row[0] - 1)]);
        }
        // highlight next row
        hightlighted_result_row[0] = hightlighted_result_row[0] + 1;
        rustbox.print(0, hightlighted_result_row[0], rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(hightlighted_result_row[0] - 1)]);
        rustbox.present();
    }

    fn hightlight_previous_row(&self, rustbox: &RustBox) {
        let mut hightlighted_result_row = self.hightlighted_result_row.lock().unwrap();
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if hightlighted_result_row[0] > 0 {
            rustbox.print(0, hightlighted_result_row[0], rustbox::RB_NORMAL, Color::White, Color::Black, &results[(hightlighted_result_row[0] - 1)]);
            if hightlighted_result_row[0] > 1 {
                // hightlight the previous row
                hightlighted_result_row[0] = hightlighted_result_row[0] - 1;
                rustbox.print(0, hightlighted_result_row[0], rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(hightlighted_result_row[0] - 1)]);
            } else {
                hightlighted_result_row[0]  = 0
            }
            rustbox.present();
        }
    }

}
