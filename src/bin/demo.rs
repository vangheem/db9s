use anyhow::Result;
use db9s::app;
use db9s::data;
use mysql;
use mysql::prelude::*;
use postgres;
use rusqlite;

fn main() -> Result<(), anyhow::Error> {
    let app = app::Application::new();
    let mut data = app.persistent_data.write().unwrap();
    let mut conns = vec![];
    for conn in data.connections.iter() {
        if ["postgres", "redis", "mysql", "sqlite", "elasticsearch"].contains(&conn.id.as_str()) {
            continue;
        }
        conns.push(conn.clone());
    }
    conns.append(&mut vec![
        data::Connection::new_with_id(
            "Postgres".to_string(),
            "postgres://postgres:postgres@localhost:5432".to_string(),
            "postgres".to_string(),
        ),
        data::Connection::new_with_id(
            "Redis".to_string(),
            "redis://localhost:6379".to_string(),
            "redis".to_string(),
        ),
        data::Connection::new_with_id(
            "MySQL".to_string(),
            "mysql://laravel:laravel@localhost:3306/laravel".to_string(),
            "mysql".to_string(),
        ),
        data::Connection::new_with_id(
            "SQLite".to_string(),
            "sqlite://testdb.sqlite".to_string(),
            "sqlite".to_string(),
        ),
        data::Connection::new_with_id(
            "ElasticSearch".to_string(),
            "elasticsearch://localhost:9200".to_string(),
            "elasticsearch".to_string(),
        ),
    ]);
    data.connections.clear();
    data.connections.append(&mut conns);
    data.save();

    let mut postgres_client = postgres::Client::connect(
        "postgres://postgres:postgres@localhost:5432",
        postgres::NoTls,
    )?;

    postgres_client.execute("DROP TABLE IF EXISTS users", &[])?;
    postgres_client.execute("DROP TABLE IF EXISTS posts", &[])?;
    postgres_client.execute("CREATE TABLE users (id INT, first TEXT, last TEXT)", &[])?;
    postgres_client.execute("CREATE TABLE posts (id INT, user_id INT, title TEXT)", &[])?;

    for i in 0..100 {
        let id = i.clone();
        let user_id = i % 10;

        postgres_client.execute(
            format!(
                "INSERT INTO posts (id, user_id, title) VALUES ({:?}, {:?}, 'Post {:?}')",
                id, user_id, i
            )
            .as_str(),
            &[],
        )?;
    }
    postgres_client.execute(
        "INSERT INTO users (id, first, last) VALUES (1, 'John', 'Doe')",
        &[],
    )?;
    postgres_client.execute(
        "INSERT INTO users (id, first, last) VALUES (2, 'Jane', 'Doe')",
        &[],
    )?;
    postgres_client.execute(
        "INSERT INTO users (id, first, last) VALUES (3, 'John', 'Smith')",
        &[],
    )?;
    postgres_client.execute(
        "INSERT INTO users (id, first, last) VALUES (4, 'Jane', 'Smith')",
        &[],
    )?;
    postgres_client.execute(
        "INSERT INTO users (id, first, last) VALUES (5, 'John', 'Jones')",
        &[],
    )?;

    let redis_conn = redis::Client::open("redis://localhost:6379")?;
    let mut redis_client = redis_conn.get_connection()?;
    // clear it all out
    redis::cmd("FLUSHALL").query::<()>(&mut redis_client)?;
    for i in 0..100 {
        let id = i.clone();
        let user_id = i % 10;

        redis::cmd("HMSET")
            .arg(format!("post:{:?}", id))
            .arg("id")
            .arg(id)
            .arg("user_id")
            .arg(user_id)
            .arg("title")
            .arg(format!("Post {:?}", i))
            .query::<()>(&mut redis_client)?;

        redis::cmd("SET")
            .arg(format!("key{:?}", i.clone()))
            .arg(format!("value{:?}", i.clone()))
            .query::<()>(&mut redis_client)?;
    }

    let sqlite_conn = rusqlite::Connection::open("testdb.sqlite")?;
    sqlite_conn.execute("DROP TABLE IF EXISTS users", [])?;
    sqlite_conn.execute("DROP TABLE IF EXISTS posts", [])?;
    sqlite_conn.execute("CREATE TABLE users (id INTEGER, first TEXT, last TEXT)", [])?;
    sqlite_conn.execute(
        "CREATE TABLE posts (id INTEGER, user_id INTEGER, title TEXT)",
        [],
    )?;
    for i in 0..100 {
        let id = i.clone();
        let user_id = i % 10;

        sqlite_conn.execute(
            format!(
                "INSERT INTO posts (id, user_id, title) VALUES ({:?}, {:?}, 'Post {:?}')",
                id, user_id, i
            )
            .as_str(),
            [],
        )?;
    }

    sqlite_conn.execute(
        "INSERT INTO users (id, first, last) VALUES (1, 'John', 'Doe')",
        [],
    )?;
    sqlite_conn.execute(
        "INSERT INTO users (id, first, last) VALUES (2, 'Jane', 'Doe')",
        [],
    )?;
    sqlite_conn.execute(
        "INSERT INTO users (id, first, last) VALUES (3, 'John', 'Smith')",
        [],
    )?;
    sqlite_conn.execute(
        "INSERT INTO users (id, first, last) VALUES (4, 'Jane', 'Smith')",
        [],
    )?;
    sqlite_conn.execute(
        "INSERT INTO users (id, first, last) VALUES (5, 'John', 'Jones')",
        [],
    )?;

    let mysql_url = "mysql://laravel:laravel@localhost:3306/laravel";
    let mysql_pool = mysql::Pool::new(mysql_url)?;

    let mut mysql_conn = mysql_pool.get_conn()?;

    mysql_conn.query_drop("DROP TABLE IF EXISTS users")?;
    mysql_conn.query_drop("DROP TABLE IF EXISTS posts")?;
    mysql_conn.query_drop("CREATE TABLE users (id INT, first TEXT, last TEXT)")?;
    mysql_conn.query_drop("CREATE TABLE posts (id INT, user_id INT, title TEXT)")?;
    for i in 0..100 {
        let id = i.clone();
        let user_id = i % 10;

        mysql_conn.query_drop(
            format!(
                "INSERT INTO posts (id, user_id, title) VALUES ({:?}, {:?}, 'Post {:?}')",
                id, user_id, i
            )
            .as_str(),
        )?;
    }

    mysql_conn.query_drop("INSERT INTO users (id, first, last) VALUES (1, 'John', 'Doe')")?;
    mysql_conn.query_drop("INSERT INTO users (id, first, last) VALUES (2, 'Jane', 'Doe')")?;
    mysql_conn.query_drop("INSERT INTO users (id, first, last) VALUES (3, 'John', 'Smith')")?;
    mysql_conn.query_drop("INSERT INTO users (id, first, last) VALUES (4, 'Jane', 'Smith')")?;
    mysql_conn.query_drop("INSERT INTO users (id, first, last) VALUES (5, 'John', 'Jones')")?;

    Ok(())
}
