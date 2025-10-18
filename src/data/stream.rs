use std::collections::{BTreeMap, HashMap};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRecord {
    id: String,
    value: StramValue,
    last_id: StreamEntryID,
}

impl StreamRecord {
    pub fn new(id: String) -> Self {
        Self {
            id,
            value: StramValue(BTreeMap::new()),
            last_id: StreamEntryID::default(),
        }
    }

    pub fn xadd(
        &mut self,
        field: String,
        value: HashMap<String, String>,
    ) -> Result<StreamEntryID, StreamRecordError> {
        let entry_id = StreamEntryID::new(&field, &self.last_id)?;
        if entry_id.ms == 0 && entry_id.seq == 0 {
            return Err(StreamRecordError::MustBeGreater00);
        }
        if entry_id <= self.last_id {
            return Err(StreamRecordError::EqualOrSmallerThanLastID);
        }

        self.value.0.insert(field, value);
        self.last_id = entry_id.clone();
        Ok(entry_id)
    }
}

#[derive(Debug, Error)]
pub enum StreamRecordError {
    #[error("Stream entry ID error: {0}")]
    StreamEntryIDError(#[from] StreamEntryIDError),
    #[error("The ID specified in XADD must be greater than 0-0")]
    MustBeGreater00,
    #[error("The ID specified in XADD is equal or smaller than the target stream top item")]
    EqualOrSmallerThanLastID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StramValue(BTreeMap<String, HashMap<String, String>>);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamEntryID {
    ms: u128,
    seq: u64,
}

impl StreamEntryID {
    pub fn new(source: &str, last_id: &StreamEntryID) -> Result<Self, StreamEntryIDError> {
        let parts: Vec<&str> = source.split('-').collect();
        if parts.len() != 2 {
            return Err(StreamEntryIDError::InvalidFormat);
        }

        let ms = parts[0]
            .parse::<u128>()
            .map_err(|_| StreamEntryIDError::InvalidFormat)?;
        let seq = if parts[1] == "*" {
            StreamEntryID::gen_seq(ms, last_id.ms, last_id.seq)
        } else {
            parts[1]
                .parse::<u64>()
                .map_err(|_| StreamEntryIDError::InvalidFormat)?
        };

        Ok(Self { ms, seq })
    }

    fn gen_seq(ms: u128, last_ms: u128, last_seq: u64) -> u64 {
        if ms == last_ms { last_seq + 1 } else { 0 }
    }
}

impl From<StreamEntryID> for String {
    fn from(value: StreamEntryID) -> Self {
        format!("{}-{}", value.ms, value.seq)
    }
}

#[derive(Debug, Error)]
pub enum StreamEntryIDError {
    #[error("Invalid stream ID format")]
    InvalidFormat,
}
