#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use deadpool_postgres::Pool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod account;
pub mod common;
pub mod middlewares;
pub mod todos;

pub struct AppState {
    pub db_pool: Pool,
    pub redis_client: Arc<Mutex<redis::aio::Connection>>,
}
