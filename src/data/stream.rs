use std::collections::{BTreeMap, HashMap};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRecord {
    id: String,
    value: StramValue,
}

impl StreamRecord {
    pub fn new(id: String) -> Self {
        Self {
            id,
            value: StramValue(BTreeMap::new()),
        }
    }

    pub fn xadd(&mut self, field: String, value: HashMap<String, String>) {
        self.value.0.insert(field, value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StramValue(BTreeMap<String, HashMap<String, String>>);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamEntryID {
    ms: u128,
    seq: u64,
}

impl StreamEntryID {
    pub fn new(source: &str) -> Result<Self, StreamEntryIDError> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum StreamEntryIDError {
    #[error("Invalid stream ID format")]
    InvalidFormat,
    #[error("The ID specified in XADD must be greater than 0-0")]
    MustBeGreater00,
    #[error("The ID specified in XADD is equal or smaller than the target stream top item")]
    EqualOrSmallerThanLastID,
}
