use crate::ui::types;
use crate::{connectiontypes::base, data::Connection};
use anyhow::Result;
use mdsn::Dsn;
extern crate reqwest;
use base64;
use std::collections::HashMap;
use std::str::FromStr; // Add missing import

pub struct ElasticSearchDatabase {
    name: String,
    dsn: Dsn,
    selections: HashMap<types::WindowTypeID, Vec<String>>,
    query: Option<String>,
}

impl base::ConnectionType for ElasticSearchDatabase {
    fn list_tables(&self) -> Result<Vec<base::Table>> {
        let indexes = self.list_indexes()?;
        Ok(indexes
            .iter()
            .map(|index| base::Table {
                id: index.clone(),
                name: index.clone(),
            })
            .collect())
    }

    fn list_indexes(&self) -> Result<Vec<String>> {
        let client = self.get_client()?;
        let url = self.get_api_url("_cat/indices");
        let response = client.get(&url).send()?;
        let body = response.text()?;
        let tables: Vec<String> = body
            .lines()
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts[2].to_string()
            })
            .collect();
        Ok(tables)
    }

    fn default_query_string(&self) -> String {
        return "{\"query\": { \"match_all\": {}}, \"size\": 50}".to_string();
    }

    fn query(&self) -> Result<base::QueryResult> {
        let mut query = self.default_query_string();
        if let Some(custom_query) = self.query.clone() {
            query = custom_query;
        }

        let client = self.get_client()?;
        let url = self.get_api_url(&format!("{}/_search", self.get_selected_index()?));
        let response = client
            .post(&url)
            .body(query.clone())
            .header(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .send()?;
        let body = response.text()?;
        let result: serde_json::Value = serde_json::from_str(&body)?;

        let mut columns = vec![];
        match self.selections.get(&types::WindowTypeID::COLUMNS) {
            Some(value) => {
                if !value.is_empty() {
                    columns = value.clone();
                }
            }
            None => {}
        }
        if columns.is_empty() {
            columns = self.list_columns()?;
        }

        let rows: Vec<base::QueryResultRow> = result["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let mut data = vec![];
                let raw_data = row["_source"].as_object().unwrap();
                for column in columns.iter() {
                    data.push(raw_data[column].to_string());
                }
                base::QueryResultRow {
                    id: row["_id"].to_string(),
                    data: data
                        .iter()
                        .map(|d| Some(d.clone().trim_matches('"').to_string()))
                        .collect(),
                }
            })
            .collect();

        Ok(base::QueryResult {
            columns: columns,
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
        let client = self.get_client()?;
        let url = self.get_api_url(&format!("{}/_mapping", self.get_selected_index()?));
        let response = client.get(&url).send()?;
        let body = response.text()?;
        let result: serde_json::Value = serde_json::from_str(&body)?;
        let columns: Vec<String> = result[self.get_selected_index()?.as_str()]["mappings"]
            .as_object()
            .unwrap()
            .values()
            .map(|v| {
                v.as_object()
                    .unwrap()
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>()
            })
            .flatten()
            .collect();
        Ok(columns)
    }
}

impl ElasticSearchDatabase {
    pub fn new(
        config: Connection,
        selections: HashMap<types::WindowTypeID, Vec<String>>,
        query: Option<String>,
    ) -> Result<Self> {
        let dsn = Dsn::from_str(&config.dsn)?;
        Ok(ElasticSearchDatabase {
            name: config.name,
            dsn: dsn,
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

    fn get_selected_index(&self) -> Result<String> {
        let mut index = self.get_selection(types::WindowTypeID::TABLES);
        if index.is_none() {
            index = self.get_selection(types::WindowTypeID::INDEXES);
            if index.is_none() {
                return Err(anyhow::anyhow!("No index selected"));
            }
        }
        Ok(index.unwrap())
    }

    fn get_client(&self) -> Result<reqwest::blocking::Client> {
        let mut builder = reqwest::blocking::Client::builder();
        if self.dsn.username.is_some() {
            let mut hm = reqwest::header::HeaderMap::new();
            hm.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!(
                    "Basic {}",
                    base64::encode(format!(
                        "{}:{}",
                        self.dsn.username.clone().unwrap(),
                        self.dsn.password.clone().unwrap(),
                    ))
                ))?,
            );
            builder = builder.default_headers(hm);
        }
        Ok(builder.build()?)
    }

    fn get_api_url(&self, path: &str) -> String {
        let url = format!("http://{}", self.dsn.addresses.clone().first().unwrap());
        format!("{}/{}", url, path)
    }
}
