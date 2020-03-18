use chrono::prelude::*;
use postgres::types::ToSql;
use postgres::{self, NoTls, Row};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Todo {
    id: i32,
    account_id: i32,
    title: String,
    body: Option<String>,
    creation_date: DateTime<Utc>,
    last_edit_date: DateTime<Utc>,
    done: bool,
}

impl Todo {
    pub fn new(
        id: i32,
        account_id: i32,
        title: String,
        body: Option<String>,
        creation_date: DateTime<Utc>,
        last_edit_date: DateTime<Utc>,
        done: bool,
    ) -> Self {
        Todo {
            id,
            account_id,
            title,
            body,
            creation_date,
            last_edit_date,
            done,
        }
    }
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
            "
            INSERT INTO todo(account_id, title, body, creation_date, last_edit_date)
            VALUES($1, $2, $3, $4, $5)
            RETURNING id, creation_date",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }

    pub fn get(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "
            SELECT
                id, account_id, title, body, creation_date, last_edit_date, done
            FROM todo
            WHERE account_id = $1
            ORDER BY last_edit_date DESC
            OFFSET $2
            LIMIT $3",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }

    pub fn edit(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "
            UPDATE todo
            SET title = COALESCE($1, title),
                body = COALESCE($2, body),
                done = COALESCE($3, done)
            WHERE account_id = $4 AND id = $5
            RETURNING id, last_edit_date",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }

    pub fn delete(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "
            DELETE FROM todo
            WHERE account_id = $1 AND id = ANY($2)
            RETURNING id",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }
}
