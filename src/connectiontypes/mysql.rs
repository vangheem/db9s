use crate::ui::types;
use crate::{connectiontypes::base, data::Connection};
use anyhow::{anyhow, Result};
use chrono::{format, DateTime, NaiveDateTime, Utc};
use log::debug;
use mysql;
use mysql::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct MySQLDatabase {
    name: String,
    dsn: String,
    selections: HashMap<types::WindowTypeID, Vec<String>>,
    query: Option<String>,
}

fn get_row_value(row: &mysql::Row, column: &str) -> Option<String> {
    let result = row
        .get::<String, _>(column)
        .or_else(|| row.get::<Uuid, _>(column).map(|value| value.to_string()))
        .or_else(|| {
            row.get_opt::<i32, _>(column)
                .map(|value| value.unwrap().to_string())
        })
        .or_else(|| {
            row.get_opt::<i64, _>(column)
                .map(|value| value.unwrap().to_string())
        })
        .or_else(|| {
            row.get_opt::<f32, _>(column)
                .map(|value| value.unwrap().to_string())
        })
        .or_else(|| {
            row.get_opt::<f64, _>(column)
                .map(|value| value.unwrap().to_string())
        })
        .or_else(|| {
            row.get_opt::<bool, _>(column)
                .map(|value| value.unwrap().to_string())
        });
    result
}

impl base::ConnectionType for MySQLDatabase {
    fn list_tables(&self) -> Result<Vec<base::Table>> {
        let tables: Vec<base::Table> =
            self.get_client()?
                .query_map("SHOW TABLES;", |table: mysql::Row| -> base::Table {
                    let tbl = table.get(0).unwrap_or("missing".to_string());
                    base::Table {
                        id: tbl.clone(),
                        name: tbl,
                    }
                })?;
        Ok(tables)
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

        let raw_rows = self.get_client()?.query_map(query, |row: mysql::Row| row)?;
        if raw_rows.is_empty() {
            return Ok(base::QueryResult {
                columns: vec![],
                rows: vec![],
            });
        }

        let columns: Vec<String> = raw_rows
            .first()
            .unwrap()
            .columns_ref()
            .iter()
            .map(|c| c.name_str().to_string())
            .collect();

        let mut rows = Vec::new();
        if columns.is_empty() {
            return Ok(base::QueryResult {
                columns: vec![],
                rows: vec![],
            });
        }
        for row in raw_rows.iter() {
            let mut row_data = Vec::new();
            for column in columns.iter() {
                row_data.push(get_row_value(row, column));
            }
            rows.push(base::QueryResultRow {
                id: get_row_value(row, columns.get(0).unwrap()).unwrap_or("missing".to_string()),
                data: row_data,
            });
        }

        Ok(base::QueryResult { columns, rows })
    }

    fn list_schemas(&self) -> Result<Vec<base::Schema>> {
        let schemas: Vec<base::Schema> = self.get_client()?.query_map(
            "SHOW schemas;",
            |schema: mysql::Row| -> base::Schema {
                let sch = schema.get(0).unwrap_or("missing".to_string());
                base::Schema {
                    id: sch.clone(),
                    name: sch,
                }
            },
        )?;
        Ok(schemas)
    }

    fn list_databases(&self) -> Result<Vec<base::DatabaseInfo>> {
        let query = "SHOW databases;";
        let raw_databases = self.get_client()?.query_map(query, |db: mysql::Row| {
            let dbv: String = db.get(0).unwrap_or("missing".to_string());
            base::DatabaseInfo {
                id: dbv.clone(),
                name: dbv,
            }
        })?;
        Ok(raw_databases)
    }

    fn list_columns(&self) -> Result<Vec<String>> {
        let table_name = &self
            .get_selection(types::WindowTypeID::TABLES)
            .unwrap_or("public".to_string());
        let query = format!("describe {}", table_name);
        let raw_columns = self.get_client()?.query_map(query, |cols: mysql::Row| {
            cols.get(0).unwrap_or("missing".to_string())
        })?;
        Ok(raw_columns)
    }
    fn list_indexes(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

impl MySQLDatabase {
    pub fn new(
        config: Connection,
        selections: HashMap<types::WindowTypeID, Vec<String>>,
        query: Option<String>,
    ) -> Result<Self> {
        Ok(MySQLDatabase {
            name: config.name,
            dsn: config.dsn,
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

    fn get_client(&self) -> Result<mysql::PooledConn> {
        let pool = mysql::Pool::new(self.dsn.as_str())?;

        let mut conn = pool.get_conn();
        if conn.is_err() {
            return Err(anyhow!("Failed to get connection from pool"));
        }
        Ok(conn.unwrap())
    }
}
