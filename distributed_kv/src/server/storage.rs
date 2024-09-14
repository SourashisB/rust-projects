use std::collections::HashMap;

pub struct Storage {
    data: HashMap<String, String>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn delete(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn list_all(&self) -> Vec<(String, String)> {
        self.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}