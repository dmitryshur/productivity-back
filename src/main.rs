#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use account::account_controllers::{account_login, account_register};
use actix_web::{middleware, web, App, HttpServer};
use postgres;
use postgres::NoTls;
use r2d2::{self, Pool};
use r2d2_postgres::PostgresConnectionManager;
use redis;
use std::sync::Arc;
use todos::todo_controllers::{todo_create, todo_delete, todo_edit, todo_get};

mod account;
mod common;
mod todos;

pub struct AppState {
    db_pool: Pool<PostgresConnectionManager<postgres::NoTls>>,
    redis_client: Arc<redis::Client>,
}

fn create_db_pool() -> Result<Pool<PostgresConnectionManager<NoTls>>, r2d2::Error> {
    let db_manager = PostgresConnectionManager::new(
        "host=localhost user=dshur dbname=productivity password=1234"
            .parse()
            .unwrap(),
        postgres::NoTls,
    );

    Pool::new(db_manager)
}

fn create_redis_client() -> redis::RedisResult<redis::Client> {
    redis::Client::open("redis://127.0.0.1:6379")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "warnings=warn,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let db_pool = match create_db_pool() {
        Ok(pool) => pool,
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);
            panic!();
        }
    };

    let redis_client = match create_redis_client() {
        Ok(client) => Arc::new(client),
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);
            panic!();
        }
    };

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(AppState {
                db_pool: db_pool.clone(),
                redis_client: redis_client.clone(),
            })
            .service(
                web::scope("/api/todo")
                    .route("/create", web::post().to(todo_create))
                    .route("/get", web::get().to(todo_get))
                    .route("/edit", web::post().to(todo_edit))
                    .route("/delete", web::post().to(todo_delete)),
            )
            .service(
                web::scope("/api/account")
                    .route("/register", web::post().to(account_register))
                    .route("/login", web::post().to(account_login)),
            )
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
}
