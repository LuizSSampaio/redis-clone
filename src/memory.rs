use std::{collections::HashMap, time::Duration};

use tokio::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Memory {
    data: HashMap<String, Value>,
}

impl Memory {
    pub fn set(&mut self, key: String, value: String, duration: Option<Duration>) {
        let value = Value {
            data: value,
            duration,
            instant: Instant::now(),
        };

        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.data.get(key).cloned().and_then(|value| {
            if let Some(duration) = value.duration
                && value.instant.elapsed() >= duration
            {
                self.data.remove(key);
                return None;
            }
            Some(value.data)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Value {
    pub data: String,
    pub duration: Option<Duration>,
    pub instant: Instant,
}
