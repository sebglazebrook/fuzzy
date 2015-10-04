extern crate rustbox;
extern crate time;
extern crate clipboard;

use rustbox::{RustBox, Key, Color};
use self::clipboard::ClipboardContext;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use fuzzy::results_view::ResultsView;
use fuzzy::search_phrase::SearchPhrase;
use fuzzy::event_service::EventService;
use std::thread;
use std::sync::mpsc::{Sender};
use std::sync::mpsc;

pub struct Terminal {
    pub rustbox: Arc<Mutex<RustBox>>,
    pub tx: Arc<Mutex<Sender<Vec<String>>>>,
    event_service: Arc<EventService>,
    search_complete: AtomicBool,
    results_view: ResultsView,
}

impl Terminal {

    pub fn new(event_service: Arc<EventService>) -> Arc<Terminal> {
        let rustbox = match RustBox::init(Default::default()) {
            Result::Ok(v) => Arc::new(Mutex::new(v)),
            Result::Err(e) => panic!("{}", e),
        };
        let (tx, _) = mpsc::channel();
        Arc::new(
            Terminal{
                rustbox: rustbox,
                event_service: event_service,
                tx: Arc::new(Mutex::new(tx)),
                search_complete: AtomicBool::new(false),
                results_view: ResultsView::new(),
            }
        )
    }

    pub fn listen_for_files(&self) {
        while !self.search_complete.load(Ordering::Relaxed) {
            let mut file_finder_events = self.event_service.file_finder_events.lock().unwrap();
            let condvar = self.event_service.file_finder_condvar.clone();

            file_finder_events = condvar.wait(file_finder_events).unwrap();
            if self.search_complete.load(Ordering::Relaxed) {
                break;
            } else {
                let events = file_finder_events.export();
                self.show_results(events.last().unwrap().clone());
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
                                self.results_view.highlight_next(&rustbox);
                            }
                            Some(Key::Ctrl('k')) => {
                                self.results_view.highlight_previous(&rustbox);
                            }
                            Some(Key::Ctrl('y')) => {
                                let mut ctx = ClipboardContext::new().unwrap();
                                let _ = ctx.set_contents(self.results_view.get_highlighted());
                                done = true;
                            }
                            Some(Key::Down) => {
                                self.results_view.highlight_next(&rustbox);
                            }
                            Some(Key::Up) => {
                                self.results_view.highlight_previous(&rustbox);
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
        self.event_service.file_finder_condvar.notify_all();
    }

    pub fn show_results(&self, results: Vec<String>) {
        self.results_view.update(self.rustbox.clone(), results);
    }

    pub fn has_highlighted_result(&self) -> bool {
        self.results_view.has_highlighted_result()
    }

    pub fn get_highlighted_result(&self) -> String {
        self.results_view.get_highlighted()
    }
}
