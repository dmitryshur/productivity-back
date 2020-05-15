#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use deadpool_postgres::{Pool, PoolError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres;

pub mod account;
pub mod common;
pub mod middlewares;
pub mod todos;

pub struct AppState {
    pub db_pool: Pool,
    pub redis_client: Arc<Mutex<redis::aio::Connection>>,
}

#[derive(Debug)]
pub enum DbErrors {
    Postgres(postgres::Error),
    Runtime,
}

impl From<PoolError> for DbErrors {
    fn from(_err: PoolError) -> DbErrors {
        return DbErrors::Runtime;
    }
}

impl From<tokio_postgres::Error> for DbErrors {
    fn from(err: tokio_postgres::Error) -> DbErrors {
        return DbErrors::Postgres(err);
    }
}
