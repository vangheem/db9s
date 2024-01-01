use std::sync::RwLock;

use crate::data;
pub struct Application {
    pub data: RwLock<data::PersistentData>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            data: RwLock::new(data::PersistentData::open()),
        }
    }

    pub fn get_connections(&self) -> Vec<data::Connection> {
        self.data.read().unwrap().connections.clone()
    }
}
