use std::{collections::VecDeque, sync::Arc, time::SystemTime};

use dashmap::DashMap;

use crate::data::record::{RecordData, RecordType};

#[derive(Debug, Default, Clone)]
pub struct Store {
    entries: Arc<DashMap<String, RecordData>>,
}

impl Store {
    pub fn set(&self, key: String, value: String, duration: Option<SystemTime>) {
        self.entries
            .insert(key, RecordData::new(RecordType::String(value), duration));
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.entries.remove(key);
                return None;
            }

            match &entry.record {
                RecordType::String(value) => Some(value.clone()),
                _ => None,
            }
        } else {
            None
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

    pub fn lpop(&self, key: &str) -> Option<String> {
        let mut entry = self.entries.get_mut(key)?;
        let RecordType::List(list) = &mut entry.record else {
            return None;
        };

        list.pop_front()
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

    pub fn llen(&self, key: &str) -> usize {
        let Some(entry) = self.entries.get(key) else {
            return 0;
        };
        let RecordType::List(list) = &entry.record else {
            return 0;
        };

        list.len()
    }
}
