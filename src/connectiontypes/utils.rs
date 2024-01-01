use super::pg;
use super::redis;
use super::sqlite;
use crate::data::Connection;
use crate::ui::types;
use crate::ui::types::WindowTypeID;
use anyhow::anyhow;
use anyhow::Result;
use mdsn::Dsn;
use std::{collections::HashMap, str::FromStr};

use super::base::ConnectionType;

const SUPPORTED_DRIVERS: [&str; 3] = ["postgres", "postgresql", "redis"];

pub fn validate_dsn(raw_dsn: String) -> (bool, String) {
    let dsn = Dsn::from_str(&raw_dsn);
    if dsn.is_err() {
        return (false, "Invalid DSN".to_string());
    }
    let dsn = dsn.unwrap();
    if !SUPPORTED_DRIVERS.contains(&dsn.driver.as_str()) {
        return (
            false,
            format!(
                "Unsupported DSN type. Allowed: ({})",
                SUPPORTED_DRIVERS.join(", ")
            ),
        );
    }
    (true, "".to_string())
}

pub fn get_connection_type(
    conn: Connection,
    selections: HashMap<types::WindowTypeID, Vec<String>>,
    custom_queries: HashMap<String, String>,
) -> Result<Box<dyn ConnectionType>> {
    let dsn = Dsn::from_str(&conn.dsn)?;
    let query = custom_queries.get(&conn.id).map(|s| s.to_string());
    if dsn.driver == "postgres" || dsn.driver == "postgresql" {
        return Ok(Box::new(pg::PostgreSQLDatabase::new(
            conn, selections, query,
        )?));
    } else if dsn.driver == "redis" {
        return Ok(Box::new(redis::RedisConnectionType::new(
            conn, selections, query,
        )?));
    } else if dsn.driver == "sqlite" {
        return Ok(Box::new(sqlite::SQLiteConnectionType::new(
            conn, selections, query,
        )?));
    }
    Err(anyhow!("Unsupported DSN type"))
}

pub fn feature_supported(conn: Connection, window_type: WindowTypeID) -> Result<bool> {
    let dsn = Dsn::from_str(&conn.dsn)?;
    if dsn.driver == "postgres" || dsn.driver == "postgresql" {
        return Ok(true); // all features supported
    } else if dsn.driver == "redis" {
        return Ok([
            WindowTypeID::DATABASES,
            WindowTypeID::CONNECTIONS,
            WindowTypeID::QUERY,
        ]
        .contains(&window_type));
    } else if dsn.driver == "sqlite" {
        return Ok([
            WindowTypeID::CONNECTIONS,
            WindowTypeID::TABLES,
            WindowTypeID::QUERY,
        ]
        .contains(&window_type));
    }
    Err(anyhow!("Unsupported DSN type"))
}
