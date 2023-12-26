use crate::ui::types::SelectionType;
use crate::{connectiontypes::base, data::Connection};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use mdsn::Dsn;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn get_row_value(row: &postgres::Row, column: &str) -> Option<String> {
    match row.try_get::<_, String>(column) {
        Ok(value) => Some(value),
        Err(_) => match row.try_get::<_, Uuid>(column) {
            Ok(value) => Some(value.to_string()),
            // Ok(value) => Some(value.to_string()),
            Err(_) => match row.try_get::<_, NaiveDateTime>(column) {
                Ok(value) => Some(value.to_string()),
                Err(_) => match row.try_get::<_, DateTime<Utc>>(column) {
                    Ok(value) => Some(value.to_string()),
                    Err(_) => match row.try_get::<_, i32>(column) {
                        Ok(value) => Some(value.to_string()),
                        Err(_) => match row.try_get::<_, i64>(column) {
                            Ok(value) => Some(value.to_string()),
                            Err(_) => match row.try_get::<_, f32>(column) {
                                Ok(value) => Some(value.to_string()),
                                Err(_) => match row.try_get::<_, f64>(column) {
                                    Ok(value) => Some(value.to_string()),
                                    Err(_) => match row.try_get::<_, bool>(column) {
                                        Ok(value) => Some(value.to_string()),
                                        Err(_) => None,
                                    },
                                },
                            },
                        },
                    },
                },
            }, // Some(format!("Failed: {:?}", column)),
        },
    }
}

pub struct PostgreSQLDatabase {
    name: String,
    client: Arc<Mutex<Client>>,
    selections: HashMap<SelectionType, Vec<String>>,
}
impl base::ConnectionType for PostgreSQLDatabase {
    fn list_tables(&self) -> Result<Vec<base::Table>> {
        let raw_tables = self.client.lock().unwrap().query(
            "
SELECT table_name
FROM information_schema.tables
WHERE table_schema = $1;",
            &[&self
                .get_selection(SelectionType::Schema)
                .unwrap_or("public".to_string())],
        )?;

        let tables: Vec<base::Table> = raw_tables
            .iter()
            .map(|t| base::Table {
                id: t.get(0),
                name: t.get(0),
            })
            .collect();

        Ok(tables)
    }
    fn query_table(
        &self,
        sort_by: Option<String>,
        limit: Option<u32>,
    ) -> Result<base::QueryResult> {
        let mut columns = vec!["*".to_string()];
        match self.selections.get(&SelectionType::Column) {
            Some(value) => {
                if !value.is_empty() {
                    columns = value.clone();
                }
            }
            None => {}
        }

        let mut query = format!(
            "SELECT {} FROM {}",
            columns.join(","),
            self.get_selection(SelectionType::Table)
                .unwrap_or("_unselected_".to_string())
        );
        if let Some(sort_by) = sort_by {
            query = format!("{} ORDER BY {}", query, sort_by);
        }
        query = format!("{} LIMIT {}", query, limit.unwrap_or(100));
        let raw_rows = self.client.lock().unwrap().query(query.as_str(), &[])?;
        if raw_rows.is_empty() {
            return Ok(base::QueryResult {
                columns: vec![],
                rows: vec![],
            });
        }

        let columns: Vec<String> = raw_rows
            .first()
            .unwrap()
            .columns()
            .iter()
            .map(|c| c.name().to_string())
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
        let query = "SELECT schema_name
FROM information_schema.schemata;
";
        let raw_schemas = self.client.lock().unwrap().query(query, &[])?;
        let rows: Vec<base::Schema> = raw_schemas
            .iter()
            .map(|r| base::Schema {
                id: r.get(0),
                name: r.get(0),
            })
            .collect();
        Ok(rows)
    }

    fn list_databases(&self) -> Result<Vec<base::DatabaseInfo>> {
        let query = "SELECT datname FROM pg_database;";
        let raw_databases = self.client.lock().unwrap().query(query, &[])?;
        let rows: Vec<base::DatabaseInfo> = raw_databases
            .iter()
            .map(|r| base::DatabaseInfo {
                id: r.get(0),
                name: r.get(0),
            })
            .collect();
        Ok(rows)
    }

    fn list_columns(&self) -> Result<Vec<String>> {
        let query = format!(
            "SELECT column_name
FROM information_schema.columns
WHERE table_schema = $1 AND table_name = $2;"
        );
        let raw_columns = self.client.lock().unwrap().query(
            query.as_str(),
            &[
                &self
                    .get_selection(SelectionType::Schema)
                    .unwrap_or("public".to_string()),
                &self
                    .get_selection(SelectionType::Table)
                    .unwrap_or("public".to_string()),
            ],
        )?;
        let rows: Vec<String> = raw_columns.iter().map(|r| r.get(0)).collect();
        Ok(rows)
    }
}

impl PostgreSQLDatabase {
    pub fn new(
        config: Connection,
        selections: HashMap<SelectionType, Vec<String>>,
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

        let client = Client::connect(conn_string, NoTls)?;
        Ok(PostgreSQLDatabase {
            name: config.name,
            client: Arc::new(Mutex::new(client)),
            selections,
        })
    }

    fn get_selection(&self, selection_type: SelectionType) -> Option<String> {
        if !self.selections.contains_key(&selection_type) {
            return None;
        }
        let value = self.selections.get(&selection_type).unwrap();
        if value.is_empty() {
            return None;
        }
        Some(value[0].clone())
    }
}
