use std::{collections::VecDeque, time::SystemTime};

use crate::data::stream::StreamRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordType {
    String(String),
    List(VecDeque<String>),
    Stream(StreamRecord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordData {
    pub record: RecordType,
    expiration: Option<SystemTime>,
}

impl RecordData {
    pub fn new(record: RecordType, expiration: Option<SystemTime>) -> Self {
        Self { record, expiration }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            return SystemTime::now() > expiration;
        }
        false
    }

    pub fn type_name(&self) -> &'static str {
        match self.record {
            RecordType::String(_) => "string",
            RecordType::List(_) => "list",
            RecordType::Stream(_) => "stream",
        }
    }
}
