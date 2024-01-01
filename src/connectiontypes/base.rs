use anyhow::Result;
pub struct Table {
    pub id: String,
    pub name: String,
}

pub struct Schema {
    pub id: String,
    pub name: String,
}

pub struct DatabaseInfo {
    pub id: String,
    pub name: String,
}

pub struct ColumnData {
    pub column: String,
    pub data: Option<String>,
}

pub struct QueryResultRow {
    pub id: String,
    pub data: Vec<Option<String>>,
}

pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryResultRow>,
}

pub trait ConnectionType: Send + Sync {
    fn list_tables(&self) -> Result<Vec<Table>>;

    fn query(&self) -> Result<QueryResult>;

    fn default_query_string(&self) -> String;

    fn list_schemas(&self) -> Result<Vec<Schema>>;

    fn list_databases(&self) -> Result<Vec<DatabaseInfo>>;

    fn list_columns(&self) -> Result<Vec<String>>;
}
