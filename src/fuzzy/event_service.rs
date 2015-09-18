use fuzzy::search_phrase::SearchPhrase;

pub struct EventService {
    search_phrases: Vec<SearchPhrase>,
}

impl EventService {

    pub fn new() -> EventService {
        EventService { search_phrases: vec![] }
    }

    pub fn trigger_search_phrase_changed(&mut self, search_phrase: SearchPhrase) {
        self.search_phrases.push(search_phrase);
    }

    pub fn fetch_all_search_query_change_events(&mut self) -> Vec<SearchPhrase> {
        let search_phrases = self.search_phrases.clone();
        self.search_phrases.clear();
        search_phrases
    }

}

