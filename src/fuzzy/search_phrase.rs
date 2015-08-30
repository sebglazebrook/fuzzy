extern crate regex;

use regex::Regex;
use std::sync::{Arc, Mutex};
use fuzzy::file_finder::FileFinder;

pub struct SearchPhrase {
    pub content: String,
    file_finder: Arc<Mutex<FileFinder>>,
}

impl SearchPhrase {

    pub fn init(file_finder: Arc<Mutex<FileFinder>>) -> SearchPhrase {
        SearchPhrase { 
            content: String::new(),
            file_finder: file_finder,
        }
    }

    pub fn update(&mut self, string: String)  {
        self.content = self.content.clone() + &string[..];
        let file_finder = self.file_finder.lock().unwrap();
        file_finder.apply_filter(self.to_regex());
    }

    pub fn delete_last(&mut self)  {
        let mut new_string = self.content.clone();
        new_string.pop();
        self.content = new_string;
        let file_finder = self.file_finder.lock().unwrap();
        file_finder.apply_filter(self.to_regex());
    }

    pub fn to_regex(&self) -> Regex {
        let mut regex_phrase = String::new();
        for character in self.content.chars() {
            regex_phrase.push_str(".*");
            regex_phrase.push(character);
        }
        Regex::new(&regex_phrase).unwrap()
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
