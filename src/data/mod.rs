use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use std::path::PathBuf;

use dirs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Connection {
    pub name: String,
    pub dsn: String,
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

impl PersistentData {
    pub fn new() -> Self {
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

    fn save(&self) {
        let settings_dir = get_settings_directory();
        let settings_path = settings_dir.join("settings.json");
        let data = serde_json::to_string(&self).unwrap();
        let data = data.as_bytes();
        let mut file = std::fs::File::create(settings_path).unwrap();
        file.write_all(data).unwrap();
    }
}
