use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::SystemTime,
};

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

    pub fn rpush(&self, key: String, value: String) -> usize {
        let mut entry = self
            .entries
            .entry(key)
            .or_insert_with(|| RecordData::new(RecordType::List(VecDeque::new()), None));

        match &mut entry.record {
            RecordType::List(list) => {
                list.push_back(value);
                list.len()
            }
            _ => 0,
        }
    }

    pub fn lpush(&self, key: String, value: String) -> usize {
        let mut entry = self
            .entries
            .entry(key)
            .or_insert_with(|| RecordData::new(RecordType::List(VecDeque::new()), None));

        match &mut entry.record {
            RecordType::List(list) => {
                list.push_front(value);
                list.len()
            }
            _ => 0,
        }
    }

    pub fn lrange(&self, key: &str, start: isize, stop: isize) -> Vec<String> {
        let Some(entry) = self.entries.get(key) else {
            return Vec::new();
        };
        let RecordType::List(list) = &entry.record else {
            return Vec::new();
        };

        let len = list.len() as isize;
        let start = if start < 0 { len + start } else { start }.max(0) as usize;
        let stop = if stop < 0 { len + stop } else { stop }.min(len - 1) as usize;

        list.range(start..=stop).map(|s| s.to_owned()).collect()
    }
}
