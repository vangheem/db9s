use super::types;
use crate::connectiontypes::base::ConnectionType;
use crate::connectiontypes::utils::get_connection_type;
use crate::data::Connection;
use crate::{app::Application, connectiontypes::utils::feature_supported};
use anyhow::Result;
use log::{error, info};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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
    pub active_window: types::WindowTypeID,
    pub selections: HashMap<types::WindowTypeID, Vec<String>>,
    pub custom_queries: HashMap<String, String>,
    pub data: HashMap<types::WindowTypeID, WindowData>,
    pub app: Arc<Application>,
    pub databases: HashMap<String, Box<dyn ConnectionType>>,
    pub dirty: bool,
    pub error: Option<String>,
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
    pub fn get_active(&self, selection: types::WindowTypeID) -> Option<String> {
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
    pub fn get_selection(&self, selection: types::WindowTypeID) -> Option<Vec<String>> {
        match self.selections.get(&selection) {
            Some(selected) => return Some(selected.clone()),
            None => None,
        }
    }

    pub fn toggle_selection(&mut self, selection: types::WindowTypeID, value: String) -> &mut Self {
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

    pub fn set_active(&mut self, selection: types::WindowTypeID, active: String) -> &mut Self {
        self.selections.insert(selection.clone(), vec![active]);
        self
    }

    pub fn del_active(&mut self, selection: types::WindowTypeID) -> &mut Self {
        if self.selections.contains_key(&selection) {
            self.selections.remove(&selection);
        }
        self
    }

    fn get_active_connection_config(&self) -> Result<Connection> {
        let active_connection = self
            .get_active(types::WindowTypeID::CONNECTIONS)
            .unwrap_or("".to_string());
        let conn_info = self
            .app
            .get_connection(&active_connection)
            .ok_or(anyhow::anyhow!("No active connection"))?;
        return Ok(conn_info);
    }
    fn get_active_connection_type(&self) -> Result<Box<dyn ConnectionType>> {
        let conn_info = self.get_active_connection_config()?;
        let conn_type = get_connection_type(
            conn_info.clone(),
            self.selections.clone(),
            self.custom_queries.clone(),
        )?;
        Ok(conn_type)
    }
    fn get_custom_query(&self) -> Option<(String, String)> {
        let cc = self.get_active_connection_config();
        if cc.is_err() {
            return None;
        }
        let cc = cc.unwrap();
        self.custom_queries
            .get(&cc.id)
            .map(|s| (cc.id, s.to_string()))
    }
}

pub struct LayoutState {
    positions: HashMap<types::WindowTypeID, i32>,
    pub inner: Arc<RwLock<LayoutStateInner>>,
}
fn update_state(
    state: Arc<RwLock<LayoutStateInner>>,
    window: types::WindowTypeID,
    data: WindowData,
) {
    let mut state = state.write().unwrap();
    state.data.clear();
    state.data.insert(window, data);
    state.dirty = true;
}

fn safely_pull_data(state: Arc<RwLock<LayoutStateInner>>) {
    let result = pull_data(Arc::clone(&state));
    let mut state = state.write().unwrap();
    state.dirty = true;
    if result.is_err() {
        let err = result.err().unwrap();
        state.error = Some(err.to_string());
        error!("Error: {:?}", err);
    }else {
        state.error = None;
    }
}

fn pull_data(state: Arc<RwLock<LayoutStateInner>>) -> Result<()> {
    let window = state.read().unwrap().active_window.clone();
    match window {
        types::WindowTypeID::CONNECTIONS => {
            let items = state
                .read()
                .unwrap()
                .app
                .get_connections()
                .iter()
                .map(|c| WindowDataRow::new(c.id.clone(), vec![c.name.clone()]))
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
        types::WindowTypeID::TABLES => {
            let db = state.read().unwrap().get_active_connection_type()?;
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
        types::WindowTypeID::SCHEMAS => {
            let db = state.read().unwrap().get_active_connection_type()?;
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
        types::WindowTypeID::DATABASES => {
            let db = state.read().unwrap().get_active_connection_type()?;
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
        types::WindowTypeID::COLUMNS => {
            let db = state.read().unwrap().get_active_connection_type()?;
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
        types::WindowTypeID::QUERY => {
            let db = state.read().unwrap().get_active_connection_type()?;
            let results = db.query()?;
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

            let cc_cq = state.read().unwrap().get_custom_query();
            if let Some((cc_id, cq)) = cc_cq {
                state
                    .read()
                    .unwrap()
                    .app
                    .persistent_data
                    .write()
                    .unwrap()
                    .add_query_history(cc_id, cq);
            }

            update_state(
                state,
                window,
                WindowData {
                    columns: results.columns,
                    rows,
                },
            );
        }
        types::WindowTypeID::HISTORY => {
            let config = state.read().unwrap().get_active_connection_config()?;
            let rows = config
                .query_history
                .iter()
                .map(|q| {
                    WindowDataRow::new(
                        q.clone(),
                        vec![q.clone().split("\n").collect::<Vec<_>>().join(" ")],
                    )
                })
                .rev()
                .collect();
            update_state(
                state,
                window,
                WindowData {
                    columns: vec!["Query".to_string()],
                    rows,
                },
            );
        }
    }
    Ok(())
}

impl LayoutState {
    pub fn new(app: Arc<Application>) -> Self {
        let ls = LayoutState {
            positions: HashMap::new(),
            inner: Arc::new(RwLock::new(LayoutStateInner {
                active_window: types::WindowTypeID::CONNECTIONS,
                selections: HashMap::new(),
                data: HashMap::new(),
                app: Arc::clone(&app),
                databases: HashMap::new(),
                custom_queries: HashMap::new(),
                dirty: true,
                error: None
            })),
        };
        safely_pull_data(Arc::clone(&ls.inner));
        ls
    }

    fn clear(&mut self) {
        self.inner.write().unwrap().data.clear();
    }

    pub fn is_dirty(&self) -> bool {
        self.inner.read().unwrap().dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.inner.write().unwrap().dirty = dirty;
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
        self.set_dirty(true);
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

    pub fn change_window(&mut self, window: types::WindowTypeID) {
        self.set_dirty(true);
        let mut state = self.inner.write().unwrap();
        state.active_window = window;
        let moved_state: Arc<RwLock<LayoutStateInner>> = Arc::clone(&self.inner);
        std::thread::spawn(move || {
            safely_pull_data(moved_state);
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
        self.set_dirty(true);
        let value = row_value.unwrap();
        let window = self.get_active_window();
        if window.selection_type() == types::ItemSelectionType::MULTI {
            self.inner
                .write()
                .unwrap()
                .toggle_selection(window.id(), value);
        } else if window.selection_type() == types::ItemSelectionType::SINGLE {
            self.inner.write().unwrap().set_active(window.id(), value);
        }
    }

    pub fn get_next_window(&self) -> Option<types::WindowTypeID> {
        /*
        Find next window for selection based on what is supported by the database
        and what is expected app flow
        */
        let config = self.get_active_connection_config();
        if config.is_err() {
            return None;
        }
        let config = config.unwrap();
        let mut window_id = Some(self.get_active_window().id());
        while window_id.is_some() {
            window_id = types::WINDOW_ORDER.get(&window_id.unwrap()).cloned();
            if window_id.is_none() {
                return None;
            }
            match feature_supported(config.clone(), window_id.unwrap()) {
                Ok(supported) => {
                    if supported {
                        return window_id;
                    }
                }
                Err(_) => {}
            }
        }
        None
    }

    pub fn select_for_next_window(&mut self) {
        let row_value = self.get_current_row_value();
        if row_value.is_none() {
            return;
        }
        self.set_dirty(true);
        let value = row_value.unwrap();
        let window = self.get_active_window();

        if window.id() == types::WindowTypeID::HISTORY {
            let cc = self.get_active_connection_config().unwrap();
            let mut data = self.inner.write().unwrap();
            data.custom_queries.insert(cc.id.clone(), value.clone());
        } else {
            self.inner.write().unwrap().set_active(window.id(), value);
        }
        for clear in window.clears() {
            self.inner.write().unwrap().del_active(clear);
        }
        let next_window = self.get_next_window();
        if let Some(next) = next_window {
            self.change_window(next);
        }
    }

    pub fn get_active_window(&self) -> types::WindowType {
        types::get_window(self.inner.read().unwrap().active_window.clone())
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

    pub fn refresh(&mut self) {
        let moved_state: Arc<RwLock<LayoutStateInner>> = Arc::clone(&self.inner);
        std::thread::spawn(move || {
            safely_pull_data(moved_state);
        });
    }

    pub fn get_connections(&self) -> Vec<Connection> {
        self.inner.read().unwrap().app.get_connections().clone()
    }

    pub fn get_active_connection_config(&self) -> Result<Connection> {
        return self.inner.read().unwrap().get_active_connection_config();
    }
    pub fn get_active_connection_type(&self) -> Result<Box<dyn ConnectionType>> {
        return self.inner.read().unwrap().get_active_connection_type();
    }

    pub fn get_current_query(&self) -> String {
        let state = self.inner.read().unwrap();
        let cc = self.get_active_connection_config().unwrap();
        let ct = self.get_active_connection_type().unwrap();
        match state.custom_queries.get(&cc.id) {
            Some(query) => query.clone(),
            None => ct.default_query_string(),
        }
    }

    pub fn update_custom_query(&mut self, query: Option<String>) {
        self.set_dirty(true);
        let cc = self.get_active_connection_config().unwrap();
        let mut data = self.inner.write().unwrap();
        if let Some(query) = query {
            data.custom_queries.insert(cc.id.clone(), query);
        } else if data.custom_queries.contains_key(&cc.id) {
            data.custom_queries.remove(&cc.id);
        }
    }
}
