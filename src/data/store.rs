use std::{collections::HashMap, sync::Arc, time::SystemTime};

use dashmap::DashMap;

use crate::data::record::{RecordData, RecordType};

#[derive(Debug, Default, Clone)]
pub struct Store {
    entries: Arc<DashMap<String, RecordData>>,
}

impl Store {
    pub fn new(entries: HashMap<String, RecordData>) -> Self {
        Self {
            entries: Arc::new(DashMap::from_iter(entries)),
        }
    }

    pub fn set(&self, key: String, value: String, duration: Option<SystemTime>) {
        self.entries
            .insert(key, RecordData::new(RecordType::String(value), duration));
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let entry = self.entries.get(key)?;
        if entry.is_expired() {
            self.entries.remove(key);
            return None;
        }

        match &entry.record {
            RecordType::String(value) => Some(value.clone()),
            _ => None,
        }
    }
}
