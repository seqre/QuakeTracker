use std::collections::HashMap;
use std::io::Cursor;

use polars::prelude::{DataFrame, JsonReader, Schema, SerReader, TimeUnit};

use crate::seismic::SeismicEvent;

#[derive(Debug, Default)]
pub struct SeismicData {
    events: HashMap<String, SeismicEvent>,
    dataframe: DataFrame,
}

impl SeismicData {
    pub fn new() -> SeismicData {
        let mut schema = Schema::default();

        use polars::prelude::DataType as DT;
        schema.insert("unid".into(), DT::String); // id
        schema.insert("lat".into(), DT::Float64); // latitude
        schema.insert("lon".into(), DT::Float64); // longitude
        schema.insert("time".into(), DT::Datetime(TimeUnit::Nanoseconds, None)); // date
        schema.insert("mag".into(), DT::Float64); // magnitude
        schema.insert("magtype".into(), DT::String); // magnitude_type
        schema.insert("depth".into(), DT::Float64); // depth
        schema.insert("evtype".into(), DT::String); // event_type
        schema.insert("flynn_region".into(), DT::String); // flynn_region

        SeismicData {
            events: HashMap::new(),
            dataframe: DataFrame::empty_with_schema(&schema),
        }
    }
    pub fn add_or_update_event(&mut self, event: SeismicEvent) {
        let json = serde_json::to_string(&event).expect("valid event can't fail");
        let json = Cursor::new(json);
        let df = JsonReader::new(json)
            .with_schema(self.dataframe.schema().into())
            .finish()
            .unwrap();
        self.dataframe = self.dataframe.vstack(&df).unwrap();

        self.events.insert(event.id.clone(), event);
    }

    pub fn add_events(&mut self, events: Vec<SeismicEvent>) {
        let json = serde_json::to_string(&events).expect("valid event can't fail");
        let json = Cursor::new(json);
        let df = JsonReader::new(json)
            .with_schema(self.dataframe.schema().into())
            .finish()
            .unwrap();
        self.dataframe = self.dataframe.vstack(&df).unwrap();

        self.events
            .extend(events.into_iter().map(|event| (event.id.clone(), event)));
    }

    pub fn get_events(&self) -> Vec<SeismicEvent> {
        self.events.values().cloned().collect()
    }

    pub fn run_on_events<F, T>(&self, func: F) -> Vec<T>
    where
        F: Fn(&SeismicEvent) -> T,
    {
        self.events.values().map(func).collect::<Vec<T>>()
    }

    pub fn get_dataframe(&self) -> &DataFrame {
        &self.dataframe
    }

    pub fn get_mut_dataframe(&mut self) -> &mut DataFrame {
        &mut self.dataframe
    }

    pub fn get_chronological_events(&self) -> Vec<SeismicEvent> {
        let mut events: Vec<SeismicEvent> = self.get_events();
        events.sort_by(|a, b| a.time.cmp(&b.time));
        events
    }
}
