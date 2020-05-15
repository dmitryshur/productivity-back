use deadpool_postgres::Pool;
use postgres::types::ToSql;
use postgres::{self, Row};
use tokio_postgres::Error as PostgresError;

pub struct AccountDbExecutor;

impl AccountDbExecutor {
    pub async fn register(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<u64, PostgresError> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let count = transaction
            .execute(
                "INSERT INTO account (email, password) VALUES ($1, crypt($2, gen_salt('bf')))",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(count)
    }

    pub async fn login(db_pool: &Pool, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, PostgresError> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let rows = transaction
            .query(
                "SELECT id FROM account WHERE email = $1 AND password = crypt($2, password)",
                params,
            )
            .await?;
        transaction.commit().await?;

        Ok(rows)
    }

    pub async fn reset(db_pool: &Pool) -> Result<(), PostgresError> {
        let mut db_client = db_pool.get().await.unwrap();
        let transaction = db_client.transaction().await?;
        let _rows = transaction.execute("DELETE FROM account", &[]).await?;
        transaction.commit().await?;

        Ok(())
    }
}
