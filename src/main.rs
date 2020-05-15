#[macro_use]
extern crate log;

use actix_web::{middleware, web, App, HttpServer};
use deadpool_postgres::{config::ConfigError, Config, Pool};
use productivity::account::account_controllers::{account_login, account_register};
use productivity::todos::todo_controllers::{todo_create, todo_delete, todo_edit, todo_get};
use productivity::{middlewares, AppState};
use redis;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

fn create_db_pool() -> Result<Pool, ConfigError> {
    let host = std::env::var("POSTGRES_HOST").expect("POSTGRES_HOST variable missing");
    let user = std::env::var("POSTGRES_USER").expect("POSTGRES_USER variable missing");
    let db = std::env::var("POSTGRES_DB").expect("POSTGRES_DB variable missing");
    let password = std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD variable missing");

    let default = Config::default();
    let cfg = Config {
        host: Some(host),
        user: Some(user),
        password: Some(password),
        dbname: Some(db),
        ..default
    };
    cfg.create_pool(NoTls)
}

async fn create_redis_client() -> redis::RedisResult<redis::aio::Connection> {
    let host = std::env::var("REDIS_HOST").expect("REDIS_HOST variable missing");
    let port = std::env::var("REDIS_PORT").expect("REDIS_PORT variable missing");
    let client = redis::Client::open(format!("redis://{}:{}", host, port))?;
    let connection = client.get_async_connection().await;

    connection
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "warnings=warn,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");

    let host = std::env::var("PRODUCTIVITY_HOST").expect("PRODUCTIVITY_HOST variable missing");
    let port = std::env::var("PRODUCTIVITY_PORT").expect("PRODUCTIVITY_PORT variable missing");

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
        let redis_client = Arc::clone(&redis_client);
        let db_pool = Pool::clone(&db_pool);

        App::new()
            .wrap(middleware::Logger::default())
            .data(AppState { db_pool, redis_client })
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
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
