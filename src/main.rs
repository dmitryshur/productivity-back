#[macro_use]
extern crate log;

use actix_web::{middleware, web, App, HttpServer};
use postgres::{self, NoTls};
use productivity::account::account_controllers::{account_login, account_register};
use productivity::todos::todo_controllers::{todo_create, todo_delete, todo_edit, todo_get};
use productivity::{middlewares, AppState};
use r2d2::{self, Pool};
use r2d2_postgres::PostgresConnectionManager;
use redis;
use std::sync::Arc;
use tokio::sync::Mutex;

fn create_db_pool() -> Result<Pool<PostgresConnectionManager<NoTls>>, r2d2::Error> {
    let db_manager = PostgresConnectionManager::new(
        "host=localhost user=dshur dbname=productivity password=1234"
            .parse()
            .unwrap(),
        postgres::NoTls,
    );

    Pool::new(db_manager)
}

async fn create_redis_client() -> redis::RedisResult<redis::aio::Connection> {
    let client = redis::Client::open("redis://127.0.0.1:6379")?;
    let connection = client.get_async_connection().await;

    connection
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
            panic!(err);
        }
    };

    let redis_client = match create_redis_client().await {
        Ok(client) => Arc::new(Mutex::new(client)),
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);
            panic!(err);
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
                    .wrap(middlewares::auth::Authentication)
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
