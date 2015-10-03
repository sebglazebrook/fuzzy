extern crate regex;

use regex::Regex;
use std::sync::Arc;
use fuzzy::event_service::EventService;

pub struct SearchPhrase {
    pub content: String,
    event_service: Arc<EventService>
}

impl SearchPhrase {

    pub fn init(event_service: Arc<EventService>) -> SearchPhrase {
        SearchPhrase { 
            content: String::new(),
            event_service: event_service
        }
    }

    pub fn from_string(string: String, event_service: Arc<EventService>) -> SearchPhrase {
        SearchPhrase { content: string, event_service: event_service }
    }

    pub fn update(&mut self, string: String)  {
        self.content = self.content.clone() + &string[..];
        self.event_service.trigger_search_phrase_changed(self.clone());
    }

    pub fn delete_last(&mut self)  {
        let mut new_string = self.content.clone();
        new_string.pop();
        self.content = new_string;
        self.event_service.trigger_search_phrase_changed(self.clone());
    }

    pub fn to_regex(&self) -> Regex {
        let mut regex_phrase = String::from("(?i)");
        for character in self.content.chars() {
            regex_phrase.push_str(".*");
            regex_phrase.push(character);
        }
        regex_phrase.push_str(".*");
        Regex::new(&regex_phrase).unwrap()
    }
}

impl Clone for SearchPhrase {

    fn clone(&self) -> Self {
        SearchPhrase::from_string(self.content.clone(), self.event_service.clone())
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
