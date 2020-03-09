use chrono::prelude::*;
use postgres::types::ToSql;
use postgres::{self, NoTls, Row};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Todo {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
    creation_date: DateTime<Utc>,
    last_edit_date: DateTime<Utc>,
    done: bool,
}

pub struct TodoDbExecutor {
    connection: PooledConnection<PostgresConnectionManager<NoTls>>,
}

impl TodoDbExecutor {
    pub fn new(connection: PooledConnection<PostgresConnectionManager<NoTls>>) -> Self {
        TodoDbExecutor { connection }
    }

    pub fn create(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "INSERT INTO todo(user_id, title, body) VALUES($1, $2, $3) RETURNING id, creation_date",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }
}
