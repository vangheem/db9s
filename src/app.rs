use std::sync::RwLock;

use crate::data;
pub struct Application {
    pub persistent_data: RwLock<data::PersistentData>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            persistent_data: RwLock::new(data::PersistentData::open()),
        }
    }

    pub fn get_connections(&self) -> Vec<data::Connection> {
        self.persistent_data.read().unwrap().connections.clone()
    }

    pub fn get_connection(&self, id: &str) -> Option<data::Connection> {
        let connections = self.persistent_data.read().unwrap().connections.clone();
        for connection in connections {
            if connection.id == id {
                return Some(connection);
            }
        }
        None
    }
}
