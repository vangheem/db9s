use crate::ui::types;
use crate::{connectiontypes::base, data::Connection};
use anyhow::Result;
use redis;
use std::collections::HashMap;

pub struct RedisConnectionType {
    name: String,
    dsn: String,
    selections: HashMap<types::WindowTypeID, Vec<String>>,
    query: Option<String>,
}
impl base::ConnectionType for RedisConnectionType {
    fn list_tables(&self) -> Result<Vec<base::Table>> {
        Ok(vec![])
    }

    fn default_query_string(&self) -> String {
        return "SCAN 0 COUNT 50".to_string();
    }

    fn query(&self) -> Result<base::QueryResult> {
        let mut con = self.get_client()?.get_connection()?;
        let mut qs = self.default_query_string();
        if let Some(custom_query) = self.query.clone() {
            qs = custom_query;
        }

        qs = qs.split("\n").collect::<Vec<_>>().join(" ");
        let qs = qs
            .split(" ")
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<&str>>();
        let mut command = redis::cmd(qs[0]);
        for arg in qs.iter().skip(1) {
            command.arg(*arg);
        }

        let mut keys = vec![];
        let mut values = vec![];
        if qs[0] == "SCAN" {
            let (_, mut chunk): (usize, Vec<String>) = command.query(&mut con)?;
            keys.append(&mut chunk);
        } else {
            keys = command.query(&mut con)?;
        }

        if ["SCAN", "KEYS"].contains(&qs[0].to_uppercase().as_str()) {
            values = redis::cmd("MGET").arg(keys.as_slice()).query(&mut con)?;
        }

        // Combine keys and values
        let rows: Vec<base::QueryResultRow> = keys
            .into_iter()
            .zip(values.into_iter())
            .map(|(key, value)| base::QueryResultRow {
                id: key.clone(),
                data: vec![Some(key), value],
            })
            .collect();

        Ok(base::QueryResult {
            columns: vec!["key".to_string(), "value".to_string()],
            rows: rows,
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

impl RedisConnectionType {
    pub fn new(
        config: Connection,
        selections: HashMap<types::WindowTypeID, Vec<String>>,
        query: Option<String>,
    ) -> Result<Self> {
        Ok(RedisConnectionType {
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

    fn get_client(&self) -> Result<redis::Client> {
        let client = redis::Client::open(self.dsn.as_str())?;
        Ok(client)
    }
}
