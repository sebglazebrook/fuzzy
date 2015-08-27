extern crate regex;

use regex::Regex;
use std::sync::{Arc, Mutex};
use std::ops::Add;
use fuzzy::terminal::Terminal;
use fuzzy::file_finder::FileFinder;

pub struct SearchPhraseController;

fn no_op_closure() -> Box<Fn(String) -> () + Send> {
    Box::new(|string: String| {})
}

pub struct SearchPhrase {
    pub content: String,
    change_listener: Box<Fn(String) -> () + Send>,
    terminal: Arc<Terminal>,
    file_finder: Arc<Mutex<FileFinder>>,
}

impl SearchPhrase {

    pub fn init(terminal: Arc<Terminal>, file_finder: Arc<Mutex<FileFinder>>) -> SearchPhrase {
        let no_op_closure = no_op_closure();
        SearchPhrase { 
            content: String::new(),
            change_listener: no_op_closure,
            terminal: terminal,
            file_finder: file_finder,
        }
    }

    pub fn from_string(string: &str, terminal: Arc<Terminal>, file_finder: Arc<Mutex<FileFinder>>) -> SearchPhrase {
        let no_op_closure = no_op_closure();
        SearchPhrase { 
            content: string.to_string(),
            change_listener: no_op_closure,
            terminal: terminal,
            file_finder: file_finder,
        }
    }

    pub fn update(&mut self, string: String)  {
        self.content = self.content.clone() + &string[..];
        (self.change_listener)(self.content.clone());
        let file_finder = self.file_finder.clone();
        let locked_file_finder = file_finder.lock().unwrap();
        locked_file_finder.apply_filter(self.to_regex());
    }

    pub fn to_regex(&self) -> Regex {
        let mut regex_phrase = String::new();
        for character in self.content.chars() {
            regex_phrase.push('.');
            regex_phrase.push('*');
            regex_phrase.push(character);
        }
        Regex::new(&regex_phrase).unwrap()
    }

    pub fn on_change(&mut self, change_listener: Box<Fn(String) -> () + Send>) {
        self.change_listener = change_listener;
    }

}

#[test]
fn by_default_it_has_no_content() {
    let search_phrase = SearchPhrase::init();
    assert_eq!(search_phrase.content, "");
}

#[test]
fn a_search_phrase_can_be_created_from_string() {
    let search_phrase = SearchPhrase::from_string("Hello there");
    assert_eq!(search_phrase.content, "Hello there");
}

#[test]
fn after_creation_the_content_can_be_updated() {
    let mut search_phrase = SearchPhrase::from_string("Hello");
    search_phrase.update(" there".to_string());
    assert_eq!(search_phrase.content, "Hello there");
}

#[test]
fn a_one_change_observer_can_be_added() {
    let mut search_phrase = SearchPhrase::from_string("Hello");
    search_phrase.on_change(Box::new(|string: String| {
        println!("Hello to you");
    }));
    search_phrase.update(" there".to_string())
}
