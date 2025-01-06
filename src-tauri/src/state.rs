use std::collections::HashMap;

use crate::seismic::SeismicEvent;

#[derive(Debug, Default)]
pub struct SeismicData {
    events: HashMap<String, SeismicEvent>,
}

impl SeismicData {
    pub fn add_or_update_event(&mut self, event: SeismicEvent) {
        self.events.insert(event.id.clone(), event);
    }

    pub fn add_events(&mut self, events: Vec<SeismicEvent>) {
        events
            .into_iter()
            .for_each(|event| self.add_or_update_event(event));
    }

    pub fn get_events(&self) -> Vec<SeismicEvent> {
        self.events.values().cloned().collect()
    }

    pub fn get_chronological_events(&self) -> Vec<SeismicEvent> {
        let mut events: Vec<SeismicEvent> = self.get_events();
        events.sort_by(|a, b| a.time.cmp(&b.time));
        events
    }
}
