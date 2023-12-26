#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowType {
    ConnectionList,
    TableList,
    Query,
    SchemaList,
    DatabaseList,
    ColumnList,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionType {
    Connection,
    Table,
    Row,
    Schema,
    Database,
    Column,
}
