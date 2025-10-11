use std::{collections::HashMap, time::Duration};

use tokio::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Memory {
    data: HashMap<String, RedisValue>,
}

impl Memory {
    pub fn set(&mut self, key: String, value: String, duration: Option<Duration>) {
        let value = RedisValue::String(StringValue {
            data: value,
            duration,
            creation_date: Instant::now(),
        });

        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.data.get(key).cloned().and_then(|value| match value {
            RedisValue::String(value) => {
                if let Some(duration) = value.duration
                    && value.creation_date.elapsed() >= duration
                {
                    self.data.remove(key);
                    return None;
                }

                Some(value.data)
            }
            RedisValue::List(_) => None,
        })
    }

    pub fn rpush(&mut self, key: String, value: String) -> usize {
        let entry = self.data.entry(key).or_insert_with(|| {
            RedisValue::List(ListValue {
                data: Vec::new(),
                creation_date: Instant::now(),
            })
        });

        if let RedisValue::List(list) = entry {
            list.data.push(value);
            list.data.len()
        } else {
            0
        }
    }

    pub fn lrange(&self, key: &str, start: isize, stop: isize) -> Option<Vec<String>> {
        self.data.get(key).cloned().and_then(|value| match value {
            RedisValue::List(list) => {
                let len = list.data.len() as isize;
                let start = if start < 0 { len + start } else { start }.max(0) as usize;
                let stop = if stop < 0 { len + stop } else { stop }.min(len - 1) as usize;

                if start > stop || start >= len as usize {
                    return Some(vec![]);
                }

                Some(list.data[start..=stop].to_vec())
            }
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RedisValue {
    String(StringValue),
    List(ListValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringValue {
    pub data: String,
    pub duration: Option<Duration>,
    pub creation_date: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListValue {
    pub data: Vec<String>,
    pub creation_date: Instant,
}
