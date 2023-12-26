use crate::data;
pub struct Application {
    pub data: data::PersistentData,
}

impl Application {
    pub fn new() -> Self {
        Application {
            data: data::PersistentData::new(),
        }
    }
}
