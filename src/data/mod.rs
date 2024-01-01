use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use std::path::PathBuf;

use dirs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub dsn: String,
    #[serde(default)]
    pub query_history: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PersistentData {
    pub connections: Vec<Connection>,
}

fn get_settings_directory() -> PathBuf {
    let path = dirs::home_dir().unwrap();
    let settings = path.join(".escuell");
    if !settings.exists() {
        std::fs::create_dir(&settings).unwrap();
    }
    settings
}

impl Connection {
    pub fn new(name: String, dsn: String) -> Self {
        Connection {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            dsn,
            query_history: vec![],
        }
    }
    pub fn new_with_id(name: String, dsn: String, id: String) -> Self {
        Connection {
            id,
            name,
            dsn,
            query_history: vec![],
        }
    }
}

impl PersistentData {
    pub fn open() -> Self {
        let settings_dir = get_settings_directory();
        let settings_path = settings_dir.join("settings.json");
        if settings_path.exists() {
            let mut file = std::fs::File::open(settings_path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            return serde_json::from_str(&contents).unwrap();
        } else {
            return PersistentData {
                connections: vec![],
            };
        }
    }

    pub fn save(&self) {
        let settings_dir = get_settings_directory();
        let settings_path = settings_dir.join("settings.json");
        let data = serde_json::to_string(&self).unwrap();
        let data = data.as_bytes();
        let mut file = std::fs::File::create(settings_path).unwrap();
        file.write_all(data).unwrap();
    }

    pub fn add_connection(&mut self, name: String, dsn: String) {
        self.connections.push(Connection::new(name, dsn));
        self.save();
    }

    pub fn add_query_history(&mut self, connection_id: String, query: String) {
        let connection = self.connections.iter_mut().find(|c| c.id == connection_id);

        if connection.is_none() {
            return;
        }
        let connection = connection.unwrap();
        if connection.query_history.contains(&query) {
            return;
        }
        connection.query_history.push(query);

        // max 100 queries
        if connection.query_history.len() > 100 {
            connection.query_history.remove(0);
        }

        self.save();
    }
}
