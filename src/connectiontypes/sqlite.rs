use crate::ui::types;
use crate::{connectiontypes::base, data::Connection};
use anyhow::Result;
use log::{debug, info};
use mdsn::Dsn;
use std::collections::HashMap;
use std::str::FromStr;

use rusqlite;

fn get_row_value(row: &rusqlite::Row, column: &str) -> Option<String> {
    let sval: Result<String, _> = row.get(column);
    if sval.is_ok() {
        return sval.ok();
    }
    let ival: Result<i32, _> = row.get(column);
    if ival.is_ok() {
        return ival.ok().map(|v| v.to_string());
    }
    None
}

pub struct SQLiteConnectionType {
    name: String,
    path: String,
    selections: HashMap<types::WindowTypeID, Vec<String>>,
    query: Option<String>,
}

impl base::ConnectionType for SQLiteConnectionType {
    fn list_tables(&self) -> Result<Vec<base::Table>> {
        let tables_query = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;";
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(tables_query)?;
        let mut rows = stmt.query([])?;

        let mut results = vec![];
        while let Some(row) = rows.next()? {
            results.push(base::Table {
                id: row.get(0)?,
                name: row.get(0)?,
            });
        }

        Ok(results)
    }

    fn default_query_string(&self) -> String {
        let mut columns = vec!["*".to_string()];
        match self.selections.get(&types::WindowTypeID::COLUMNS) {
            Some(value) => {
                if !value.is_empty() {
                    columns = value.clone();
                }
            }
            None => {}
        }
        let query = format!(
            "SELECT {} FROM {}",
            columns.join(","),
            self.get_selection(types::WindowTypeID::TABLES)
                .unwrap_or("_unselected_".to_string())
        );
        format!("{} LIMIT {}", query, 50)
    }

    fn query(&self) -> Result<base::QueryResult> {
        let mut query = self.default_query_string();
        if let Some(custom_query) = self.query.clone() {
            query = custom_query;
        }
        debug!("List tables query: {:?}", query);

        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let mut raw_rows = stmt.query([])?;

        if columns.is_empty() {
            return Ok(base::QueryResult {
                columns: vec![],
                rows: vec![],
            });
        }

        let mut results = vec![];
        let id_colm = columns.first().unwrap();
        while let Some(row) = raw_rows.next()? {
            results.push(base::QueryResultRow {
                id: get_row_value(row, id_colm.as_str()).unwrap_or("missing".to_string()),
                data: columns.iter().map(|c| get_row_value(row, c)).collect(),
            });
        }

        if results.is_empty() {
            return Ok(base::QueryResult {
                columns: vec![],
                rows: vec![],
            });
        }

        Ok(base::QueryResult {
            columns,
            rows: results,
        })
    }

    fn list_schemas(&self) -> Result<Vec<base::Schema>> {
        Ok(vec![])
    }

    fn list_databases(&self) -> Result<Vec<base::DatabaseInfo>> {
        Ok(vec![])
    }

    fn list_columns(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

impl SQLiteConnectionType {
    pub fn new(
        config: Connection,
        selections: HashMap<types::WindowTypeID, Vec<String>>,
        query: Option<String>,
    ) -> Result<Self> {
        let dsn = Dsn::from_str(&config.dsn)?;
        let addr = dsn.addresses.first().unwrap();
        let conn_string = &format!(
            "host={} port={} dbname={} user={} password={} ",
            addr.clone().host.unwrap_or("localhost".to_string()),
            addr.port.unwrap_or(5432),
            dsn.subject.unwrap_or("postgres".to_string()),
            dsn.username.unwrap_or("postgres".to_string()),
            dsn.password.unwrap_or("".to_string())
        );

        Ok(SQLiteConnectionType {
            name: config.name,
            path: addr.clone().host.unwrap(),
            selections,
            query,
        })
    }

    fn get_selection(&self, selection_type: types::WindowTypeID) -> Option<String> {
        if !self.selections.contains_key(&selection_type) {
            return None;
        }
        let value = self.selections.get(&selection_type).unwrap();
        if value.is_empty() {
            return None;
        }
        Some(value[0].clone())
    }

    fn get_connection(&self) -> Result<rusqlite::Connection> {
        info!("Opening connection to {}", self.path.clone());
        return Ok(rusqlite::Connection::open(self.path.clone())?);
    }
}
