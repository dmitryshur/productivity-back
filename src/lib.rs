#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use r2d2::{self, Pool};
use r2d2_postgres::PostgresConnectionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod account;
pub mod common;
pub mod middlewares;
pub mod todos;

pub struct AppState {
    pub db_pool: Pool<PostgresConnectionManager<postgres::NoTls>>,
    pub redis_client: Arc<Mutex<redis::aio::Connection>>,
}
