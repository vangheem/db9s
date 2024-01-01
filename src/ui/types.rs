use core::panic;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum ItemSelectionType {
    NONE,
    SINGLE,
    MULTI,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum WindowTypeID {
    CONNECTIONS,
    TABLES,
    SCHEMAS,
    DATABASES,
    COLUMNS,
    QUERY,
    HISTORY,
}

#[derive(Clone, Debug)]
pub struct WindowType {
    id: WindowTypeID,
    title: String,
    selection_type: ItemSelectionType,
    clears: Vec<WindowTypeID>,
}

impl PartialEq for WindowType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for WindowType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<&Lazy<WindowType>> for WindowType {
    fn from(lazy: &Lazy<WindowType>) -> Self {
        WindowType::new(
            lazy.id(),
            lazy.title(),
            lazy.selection_type(),
            lazy.clears(),
        )
    }
}

impl WindowType {
    pub fn new(
        id: WindowTypeID,
        title: String,
        selection_type: ItemSelectionType,
        clears: Vec<WindowTypeID>,
    ) -> Self {
        WindowType {
            id,
            title: title,
            selection_type,
            clears,
        }
    }
    pub fn id(&self) -> WindowTypeID {
        self.id.clone()
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn selection_type(&self) -> ItemSelectionType {
        self.selection_type.clone()
    }
    pub fn clears(&self) -> Vec<WindowTypeID> {
        self.clears.clone()
    }
}

pub const WINDOW_ORDER: Lazy<HashMap<WindowTypeID, WindowTypeID>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(WindowTypeID::CONNECTIONS, WindowTypeID::TABLES);
    map.insert(WindowTypeID::DATABASES, WindowTypeID::TABLES);
    map.insert(WindowTypeID::SCHEMAS, WindowTypeID::TABLES);
    map.insert(WindowTypeID::TABLES, WindowTypeID::QUERY);
    map.insert(WindowTypeID::COLUMNS, WindowTypeID::QUERY);
    map.insert(WindowTypeID::HISTORY, WindowTypeID::QUERY);
    map
});

pub const CONNECTION_LIST: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::CONNECTIONS,
        "Connections".to_string(),
        ItemSelectionType::SINGLE,
        vec![
            WindowTypeID::TABLES,
            WindowTypeID::SCHEMAS,
            WindowTypeID::DATABASES,
            WindowTypeID::COLUMNS,
        ],
    )
});
pub const TABLE_LIST: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::TABLES,
        "Tables".to_string(),
        ItemSelectionType::SINGLE,
        vec![WindowTypeID::QUERY, WindowTypeID::COLUMNS],
    )
});
pub const QUERY: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::QUERY,
        "Query".to_string(),
        ItemSelectionType::NONE,
        vec![],
    )
});
pub const SCHEMA_LIST: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::SCHEMAS,
        "Schemas".to_string(),
        ItemSelectionType::SINGLE,
        vec![WindowTypeID::TABLES, WindowTypeID::COLUMNS],
    )
});
pub const DATABASE_LIST: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::DATABASES,
        "Databases".to_string(),
        ItemSelectionType::SINGLE,
        vec![
            WindowTypeID::TABLES,
            WindowTypeID::SCHEMAS,
            WindowTypeID::COLUMNS,
        ],
    )
});
pub const COLUMN_LIST: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::COLUMNS,
        "Columns".to_string(),
        ItemSelectionType::MULTI,
        vec![],
    )
});
pub const HISTORY: Lazy<WindowType> = Lazy::new(|| {
    WindowType::new(
        WindowTypeID::HISTORY,
        "Query History".to_string(),
        ItemSelectionType::SINGLE,
        vec![],
    )
});

pub const WINDOW_TYPES: [Lazy<WindowType>; 7] = [
    CONNECTION_LIST,
    TABLE_LIST,
    QUERY,
    SCHEMA_LIST,
    DATABASE_LIST,
    COLUMN_LIST,
    HISTORY,
];

pub fn get_window(id: WindowTypeID) -> WindowType {
    for window_type in WINDOW_TYPES.iter() {
        if window_type.id() == id {
            return window_type.into();
        }
    }
    panic!("No window type found for id: {:?}", id);
}
