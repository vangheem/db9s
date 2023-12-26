use crate::connectiontypes::pg;
use crate::{app::Application, connectiontypes::base::ConnectionType, ui::types::WindowType};
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};

use super::types::SelectionType;

#[derive(Clone)]
pub struct WindowDataRow {
    pub id: String,
    pub data: Vec<String>,
}
#[derive(Clone)]
pub struct WindowData {
    pub columns: Vec<String>,
    pub rows: Vec<WindowDataRow>,
}

pub struct LayoutStateInner {
    pub active_window: WindowType,
    pub selections: HashMap<SelectionType, Vec<String>>,
    pub data: HashMap<WindowType, WindowData>,
    pub app: Arc<Application>,
    pub databases: HashMap<String, Box<dyn ConnectionType>>,
}

impl WindowDataRow {
    pub fn new(id: String, data: Vec<String>) -> Self {
        WindowDataRow { id, data }
    }
    pub fn from_string(data: String) -> Self {
        WindowDataRow {
            id: data.clone(),
            data: vec![data],
        }
    }
    pub fn from_str(data: &str) -> Self {
        WindowDataRow {
            id: data.to_string(),
            data: vec![data.to_string()],
        }
    }
}
impl LayoutStateInner {
    pub fn get_active(&self, selection: SelectionType) -> Option<String> {
        match self.selections.get(&selection) {
            Some(selected) => {
                if selected.len() > 0 {
                    return Some(selected[0].clone());
                } else {
                    None
                }
            }
            None => None,
        }
    }
    pub fn get_selection(&self, selection: SelectionType) -> Option<Vec<String>> {
        match self.selections.get(&selection) {
            Some(selected) => return Some(selected.clone()),
            None => None,
        }
    }

    pub fn toggle_selection(&mut self, selection: SelectionType, value: String) -> &mut Self {
        if !self.selections.contains_key(&selection) {
            self.selections.insert(selection.clone(), vec![]);
        }
        if self.selections.get(&selection).unwrap().contains(&value) {
            let index = self
                .selections
                .get(&selection)
                .unwrap()
                .iter()
                .position(|x| *x == value)
                .unwrap();
            self.selections.get_mut(&selection).unwrap().remove(index);
        } else {
            self.selections
                .get_mut(&selection)
                .unwrap()
                .push(value.clone());
        }
        self
    }

    pub fn set_active(&mut self, selection: SelectionType, active: String) -> &mut Self {
        self.selections.insert(selection.clone(), vec![active]);
        self
    }

    pub fn del_active(&mut self, selection: SelectionType) -> &mut Self {
        if self.selections.contains_key(&selection) {
            self.selections.remove(&selection);
        }
        self
    }
}

pub struct LayoutState {
    positions: HashMap<WindowType, i32>,
    pub inner: Arc<RwLock<LayoutStateInner>>,
}

fn get_active_connection(state: Arc<RwLock<LayoutStateInner>>) -> Result<pg::PostgreSQLDatabase> {
    let active_connection = state
        .read()
        .unwrap()
        .get_active(SelectionType::Connection)
        .unwrap_or("".to_string());
    let connections = state.read().unwrap().app.data.connections.clone();
    let conn = connections.iter().find(|c| c.name == active_connection);
    if conn.is_none() {
        return Err(anyhow::anyhow!("No active connection"));
    }
    let conn = conn.unwrap();
    let db = pg::PostgreSQLDatabase::new(conn.clone(), state.read().unwrap().selections.clone())?;
    Ok(db)
}

fn update_state(state: Arc<RwLock<LayoutStateInner>>, window: WindowType, data: WindowData) {
    let mut state = state.write().unwrap();
    state.data.clear();
    state.data.insert(window, data);
}

fn pull_data(state: Arc<RwLock<LayoutStateInner>>) -> Result<()> {
    let window = state.read().unwrap().active_window.clone();
    match window {
        WindowType::ConnectionList => {
            let items = state
                .read()
                .unwrap()
                .app
                .data
                .connections
                .iter()
                .map(|c| WindowDataRow::from_str(&c.name))
                .collect();
            let mut state = state.write().unwrap();
            state.data.clear();
            state.data.insert(
                window,
                WindowData {
                    columns: vec!["Name".to_string()],
                    rows: items,
                },
            );
        }
        WindowType::TableList => {
            let db = get_active_connection(Arc::clone(&state))?;
            update_state(
                state,
                window,
                WindowData {
                    columns: vec!["Name".to_string()],
                    rows: db
                        .list_tables()?
                        .into_iter()
                        .map(|t| WindowDataRow::from_str(&t.name))
                        .collect(),
                },
            );
        }
        WindowType::SchemaList => {
            let db = get_active_connection(Arc::clone(&state))?;
            update_state(
                state,
                window,
                WindowData {
                    columns: vec!["Name".to_string()],
                    rows: db
                        .list_schemas()?
                        .into_iter()
                        .map(|t| WindowDataRow::from_str(&t.name))
                        .collect(),
                },
            );
        }
        WindowType::DatabaseList => {
            let db = get_active_connection(Arc::clone(&state))?;
            update_state(
                state,
                window,
                WindowData {
                    columns: vec!["Name".to_string()],
                    rows: db
                        .list_databases()?
                        .into_iter()
                        .map(|t| WindowDataRow::from_str(&t.name))
                        .collect(),
                },
            );
        }
        WindowType::ColumnList => {
            let db = get_active_connection(Arc::clone(&state))?;
            update_state(
                state,
                window,
                WindowData {
                    columns: vec!["Name".to_string()],
                    rows: db
                        .list_columns()?
                        .into_iter()
                        .map(|t| WindowDataRow::from_str(&t))
                        .collect(),
                },
            );
        }
        WindowType::Query => {
            let db = get_active_connection(Arc::clone(&state))?;
            let results = db.query_table(None, None)?;
            let rows = results
                .rows
                .iter()
                .map(|r| {
                    WindowDataRow::new(
                        r.id.clone(),
                        r.data
                            .iter()
                            .map(|c| c.clone().unwrap_or("".to_string()))
                            .collect(),
                    )
                })
                .collect();
            update_state(
                state,
                window,
                WindowData {
                    columns: results.columns,
                    rows,
                },
            );
        }
    }
    Ok(())
}

fn data_poller(state: Arc<RwLock<LayoutStateInner>>) {
    loop {
        pull_data(Arc::clone(&state));
        sleep(Duration::from_millis(1000));
    }
}

impl LayoutState {
    pub fn new(app: Arc<Application>) -> Self {
        let ls = LayoutState {
            positions: HashMap::new(),
            inner: Arc::new(RwLock::new(LayoutStateInner {
                active_window: WindowType::ConnectionList,
                selections: HashMap::new(),
                data: HashMap::new(),
                app: Arc::clone(&app),
                databases: HashMap::new(),
            })),
        };
        pull_data(Arc::clone(&ls.inner));
        let moved_state = Arc::clone(&ls.inner);
        // std::thread::spawn(move || data_poller(moved_state));
        ls
    }

    fn clear(&mut self) {
        self.inner.write().unwrap().data.clear();
    }

    pub fn get_position(&self) -> i32 {
        match self
            .positions
            .get(&self.inner.read().unwrap().active_window)
        {
            Some(pos) => pos.clone(),
            None => -1,
        }
    }

    pub fn set_position(&mut self, pos: i32) {
        let aw = self.inner.read().unwrap().active_window;
        if pos < 0 {
            self.positions.insert(aw, -1);
            return;
        }
        let wd = self.get_window_data();
        if pos > wd.rows.len() as i32 - 1 {
            self.positions.insert(aw, wd.rows.len() as i32 - 1);
            return;
        }
        self.positions
            .insert(self.inner.read().unwrap().active_window, pos);
    }

    pub fn change_window(&mut self, window: WindowType) {
        let mut state = self.inner.write().unwrap();
        state.active_window = window;
        let moved_state: Arc<RwLock<LayoutStateInner>> = Arc::clone(&self.inner);
        std::thread::spawn(move || {
            let result = pull_data(moved_state);
            if result.is_err() {
                println!("Error: {:?}", result.err().unwrap());
            }
        });
    }

    pub fn get_current_row_value(&self) -> Option<String> {
        let pos = self.get_position();
        let wd = self.get_window_data();
        if pos < 0 || pos > wd.rows.len() as i32 - 1 {
            return None;
        }
        Some(wd.rows[pos as usize].id.clone())
    }

    pub fn select_current(&mut self) {
        let row_value = self.get_current_row_value();
        if row_value.is_none() {
            return;
        }
        let value = row_value.unwrap();
        let window = self.get_active_window();
        match window {
            WindowType::ConnectionList => {}
            WindowType::TableList => {}
            WindowType::SchemaList => {}
            WindowType::DatabaseList => {}
            WindowType::ColumnList => {
                self.inner
                    .write()
                    .unwrap()
                    .toggle_selection(SelectionType::Column, value);
            }
            WindowType::Query => {}
        }
    }

    pub fn select_for_next_window(&mut self) {
        let row_value = self.get_current_row_value();
        if row_value.is_none() {
            return;
        }
        let value = row_value.unwrap();
        let window = self.get_active_window();
        match window {
            WindowType::ConnectionList => {
                self.inner
                    .write()
                    .unwrap()
                    .set_active(SelectionType::Connection, value)
                    .del_active(SelectionType::Table)
                    .del_active(SelectionType::Schema)
                    .del_active(SelectionType::Database)
                    .del_active(SelectionType::Column);
                self.change_window(WindowType::TableList);
            }
            WindowType::TableList => {
                self.inner
                    .write()
                    .unwrap()
                    .set_active(SelectionType::Table, value)
                    .del_active(SelectionType::Column);
                self.change_window(WindowType::Query);
            }
            WindowType::SchemaList => {
                self.inner
                    .write()
                    .unwrap()
                    .set_active(SelectionType::Schema, value)
                    .del_active(SelectionType::Table)
                    .del_active(SelectionType::Column);
                self.change_window(WindowType::TableList);
            }
            WindowType::DatabaseList => {
                self.inner
                    .write()
                    .unwrap()
                    .set_active(SelectionType::Database, value)
                    .del_active(SelectionType::Table)
                    .del_active(SelectionType::Schema)
                    .del_active(SelectionType::Column);
                self.change_window(WindowType::TableList);
            }
            WindowType::ColumnList => {}
            WindowType::Query => {}
        }
    }

    pub fn get_active_window(&self) -> WindowType {
        self.inner.read().unwrap().active_window.clone()
    }

    pub fn get_window_data(&self) -> WindowData {
        let state = self.inner.read().unwrap();
        match state.data.get(&state.active_window) {
            Some(items) => items.clone(),
            None => WindowData {
                columns: vec![],
                rows: vec![],
            },
        }
    }
}
