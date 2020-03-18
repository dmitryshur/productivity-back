use postgres::types::ToSql;
use postgres::{self, NoTls, Row};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;

#[derive(Debug)]
pub enum DbErrors {
    Postgres(postgres::Error),
    Runtime,
}

pub struct AccountDbExecutor {
    connection: PooledConnection<PostgresConnectionManager<NoTls>>,
}

impl AccountDbExecutor {
    pub fn new(connection: PooledConnection<PostgresConnectionManager<NoTls>>) -> Self {
        AccountDbExecutor { connection }
    }

    pub fn register(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<u64, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let count = transaction.execute(
            "INSERT INTO account (email, password) VALUES ($1, crypt($2, gen_salt('bf')))",
            params,
        )?;
        transaction.commit()?;

        Ok(count)
    }

    pub fn login(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        unimplemented!()
    }
}
