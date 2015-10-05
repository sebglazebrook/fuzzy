extern crate rustbox;
extern crate time;
extern crate clipboard;

use rustbox::{RustBox, Color};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ResultsView {
    results: Mutex<Vec<String>>,
    highlighted_result_row: AtomicUsize,
    number_of_results: AtomicUsize,
}

impl ResultsView {

    pub fn new() -> ResultsView {
        ResultsView { 
            results: Mutex::new(vec![]),
            highlighted_result_row: AtomicUsize::new(0),
            number_of_results: AtomicUsize::new(0),
        }
    }

    pub fn update(&self, rustbox: Arc<Mutex<RustBox>>, results: Vec<String>) {
        self.highlighted_result_row.store(0, Ordering::Relaxed);
        self.clear(rustbox.clone());
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

    pub fn highlight_next(&self, rustbox: &RustBox) {
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if self.highlighted_result_row.load(Ordering::Relaxed) > 0 {
            rustbox.print(0, self.highlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::White, Color::Black, &results[(self.highlighted_result_row.load(Ordering::Relaxed) - 1)]);
        }
        // highlight next row
        self.highlighted_result_row.fetch_add(1, Ordering::Relaxed);
        rustbox.print(0, self.highlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(self.highlighted_result_row.load(Ordering::Relaxed) - 1)]);
        rustbox.present();
    }

    pub fn highlight_previous(&self, rustbox: &RustBox) {
        let results = self.results.lock().unwrap();
        // unhighlight the current row
        if self.highlighted_result_row.load(Ordering::Relaxed) > 0 {
            rustbox.print(0, self.highlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::White, Color::Black, &results[(self.highlighted_result_row.load(Ordering::Relaxed) - 1)]);
            if self.highlighted_result_row.load(Ordering::Relaxed) > 1 {
                // hightlight the previous row
                self.highlighted_result_row.fetch_sub(1, Ordering::Relaxed);
                rustbox.print(0, self.highlighted_result_row.load(Ordering::Relaxed), rustbox::RB_NORMAL, Color::Magenta, Color::Black, &results[(self.highlighted_result_row.load(Ordering::Relaxed) - 1)]);
            } else {
                self.highlighted_result_row.store(0, Ordering::Relaxed)
            }
            rustbox.present();
        }
    }

    pub fn has_highlighted_result(&self) -> bool {
        self.highlighted_result_row.load(Ordering::Relaxed) > 0
    }

    pub fn get_highlighted(&self) -> String {
        let index = self.highlighted_result_row.load(Ordering::Relaxed);
        index.to_string();
        self.results.lock().unwrap()[index - 1].clone()
    }

    // -------- private methods ---------- //
    
    fn clear(&self, rustbox: Arc<Mutex<RustBox>>) {
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
}

