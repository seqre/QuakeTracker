mod client;
mod seismic;

use std::collections::HashMap;
use std::error::Error;

use tauri::async_runtime::Mutex;
use tauri::{App, Manager, Runtime};

use crate::seismic::SeismicEvent;

pub type AppState = Mutex<SeismicData>;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // https://tauri.app/plugin/logging/
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            client::get_seismic_events,
            client::listen_to_seismic_events
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn Error>> {
    app.manage(Mutex::new(SeismicData::default()));
    Ok(())
}
