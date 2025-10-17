use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::SystemTime,
};

use dashmap::DashMap;
use tokio::sync::{RwLock, oneshot};

use crate::data::{
    record::{RecordData, RecordType},
    stream::StreamRecord,
};

#[derive(Debug, Default, Clone)]
pub struct Store {
    entries: Arc<DashMap<String, RecordData>>,
    waiters: Arc<RwLock<HashMap<String, VecDeque<oneshot::Sender<()>>>>>,
}

impl Store {
    async fn notify_waiters(&self, key: &str) {
        let mut waiters = self.waiters.write().await;
        let Some(queue) = waiters.get_mut(key) else {
            return;
        };

        while let Some(waiter) = queue.pop_front() {
            if waiter.send(()).is_ok() {
                return;
            }
        }
    }

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

    pub async fn rpush(&self, key: String, value: String) -> usize {
        let mut entry = self
            .entries
            .entry(key.clone())
            .or_insert_with(|| RecordData::new(RecordType::List(VecDeque::new()), None));

        match &mut entry.record {
            RecordType::List(list) => {
                list.push_back(value);

                self.notify_waiters(&key).await;

                list.len()
            }
            _ => 0,
        }
    }

    pub async fn lpush(&self, key: String, value: String) -> usize {
        let mut entry = self
            .entries
            .entry(key.clone())
            .or_insert_with(|| RecordData::new(RecordType::List(VecDeque::new()), None));

        match &mut entry.record {
            RecordType::List(list) => {
                list.push_front(value);

                self.notify_waiters(&key).await;

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

    pub async fn blpop(&self, key: &str, deadline: Option<SystemTime>) -> Option<(String, String)> {
        loop {
            if let Some(value) = self.lpop(key) {
                return Some((key.to_string(), value));
            }

            let receiver = {
                let mut waiters = self.waiters.write().await;
                let queue = waiters.entry(key.to_string()).or_default();
                let (sender, receiver) = oneshot::channel();
                queue.push_back(sender);
                receiver
            };

            if let Some(dl) = deadline {
                if SystemTime::now() >= dl {
                    return None;
                }

                let remaining = dl.duration_since(SystemTime::now()).ok()?;
                if tokio::time::timeout(remaining, receiver).await.is_err() {
                    return None;
                }
            } else {
                receiver.await.ok()?;
            }
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

    pub fn llen(&self, key: &str) -> usize {
        let Some(entry) = self.entries.get(key) else {
            return 0;
        };
        let RecordType::List(list) = &entry.record else {
            return 0;
        };

        list.len()
    }

    pub fn type_of(&self, key: &str) -> &'static str {
        let Some(entry) = self.entries.get(key) else {
            return "none";
        };

        entry.type_name()
    }

    pub fn xadd(
        &self,
        key: String,
        field: String,
        value: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let mut entry = self.entries.entry(key.clone()).or_insert_with(|| {
            RecordData::new(RecordType::Stream(StreamRecord::new(key.clone())), None)
        });

        if let RecordType::Stream(stream_record) = &mut entry.record {
            stream_record.xadd(field, value)?;
        }
        Ok(())
    }
}
