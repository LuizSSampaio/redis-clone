use std::collections::{BTreeMap, HashMap};

use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StreamRecord {
    value: StramValue,
    last_id: StreamEntryID,
}

impl StreamRecord {
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

    pub fn xrange(&self, start: String, end: String) -> Result<StramValue, StreamRecordError> {
        let start = if start == "-" {
            self.value
                .0
                .first_key_value()
                .map(|(k, _)| StreamEntryID::parse_id(k, &self.last_id))
                .unwrap_or(Ok(StreamEntryID { ms: 0, seq: 0 }))?
        } else {
            start.try_into()?
        };
        let end = if end == "+" {
            self.value
                .0
                .last_key_value()
                .map(|(k, _)| StreamEntryID::parse_id(k, &self.last_id))
                .unwrap_or(Ok(StreamEntryID {
                    ms: u128::MAX,
                    seq: u64::MAX,
                }))?
        } else {
            end.try_into()?
        };

        let mut result = BTreeMap::new();
        for (key, value) in &self.value.0 {
            let entry_id = StreamEntryID::parse_id(key, &self.last_id)?;
            if entry_id >= start && entry_id <= end {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(StramValue(result))
    }

    pub fn xread(&self, id: String) -> Result<StramValue, StreamRecordError> {
        let id = id.try_into()?;

        let mut result = BTreeMap::new();
        for value in &self.value.0 {
            let entry_id = StreamEntryID::parse_id(value.0, &self.last_id)?;
            if entry_id > id {
                result.insert(value.0.clone(), value.1.clone());
            }
        }
        Ok(StramValue(result))
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StramValue(pub BTreeMap<String, HashMap<String, String>>);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamEntryID {
    ms: u128,
    seq: u64,
}

impl StreamEntryID {
    pub fn new(source: &str, last_id: &StreamEntryID) -> Result<Self, StreamEntryIDError> {
        if source == "*" {
            return Ok(Self::gen_id(last_id));
        }

        Self::parse_id(source, last_id)
    }

    fn parse_id(source: &str, last_id: &StreamEntryID) -> Result<Self, StreamEntryIDError> {
        let parts: Vec<&str> = source.split('-').collect();
        if parts.len() != 2 {
            return Err(StreamEntryIDError::InvalidFormat);
        }

        let ms = parts[0]
            .parse::<u128>()
            .map_err(|_| StreamEntryIDError::InvalidFormat)?;
        let seq = if parts[1] == "*" {
            Self::gen_seq(ms, last_id.ms, last_id.seq)
        } else {
            parts[1]
                .parse::<u64>()
                .map_err(|_| StreamEntryIDError::InvalidFormat)?
        };

        Ok(Self { ms, seq })
    }

    fn gen_id(last_id: &StreamEntryID) -> Self {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let seq = Self::gen_seq(ms, last_id.ms, last_id.seq);
        Self { ms, seq }
    }

    fn gen_seq(ms: u128, last_ms: u128, last_seq: u64) -> u64 {
        if ms == last_ms { last_seq + 1 } else { 0 }
    }
}

impl TryFrom<String> for StreamEntryID {
    type Error = StreamEntryIDError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 2 {
            return Err(StreamEntryIDError::InvalidFormat);
        }

        let ms = parts[0]
            .parse::<u128>()
            .map_err(|_| StreamEntryIDError::InvalidFormat)?;
        let seq = parts[1]
            .parse::<u64>()
            .map_err(|_| StreamEntryIDError::InvalidFormat)?;

        Ok(Self { ms, seq })
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
