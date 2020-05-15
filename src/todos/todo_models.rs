use chrono::prelude::*;
use deadpool_postgres::Pool;
use postgres::types::ToSql;
use postgres::{self, Row};
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

pub struct TodoDbExecutor;

impl TodoDbExecutor {
    pub async fn create(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let rows = transaction
            .query(
                "
            INSERT INTO todo(account_id, title, body, creation_date, last_edit_date)
            VALUES($1, $2, $3, $4, $5)
            RETURNING id, creation_date",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(rows)
    }

    pub async fn get(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let rows = transaction
            .query(
                "
            SELECT
                id, account_id, title, body, creation_date, last_edit_date, done
            FROM todo
            WHERE account_id = $1
            ORDER BY last_edit_date DESC
            OFFSET $2
            LIMIT $3",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(rows)
    }

    pub async fn edit(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let rows = transaction
            .query(
                "
            UPDATE todo
            SET title = COALESCE($1, title),
                body = COALESCE($2, body),
                done = COALESCE($3, done),
                last_edit_date = $4
            WHERE account_id = $5 AND id = $6
            RETURNING id, last_edit_date",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(rows)
    }

    pub async fn delete(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let rows = transaction
            .query(
                "
            DELETE FROM todo
            WHERE account_id = $1 AND id = ANY($2)
            RETURNING id",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(rows)
    }

    pub async fn reset(db_pool: &Pool) -> Result<(), postgres::Error> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let _rows = transaction.execute("DELETE FROM todo", &[]).await?;
        transaction.commit().await?;

        Ok(())
    }
}
