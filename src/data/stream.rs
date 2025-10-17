use std::collections::{BTreeMap, HashMap};

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
