use actix_http::{body::MessageBody, http::header::HeaderMap};
use actix_web::{dev::ServiceResponse, test, web};
use deadpool_postgres::{config::ConfigError, Config, Pool};
use productivity::{account::account_controllers, middlewares, todos::todo_controllers};
use redis;
use redis::ConnectionLike;
use regex::Regex;
use serde_json::Value;
use std::{sync::mpsc, thread, time::Duration};
use tokio_postgres::NoTls;

#[cfg(test)]
pub fn test_config_app(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/todo")
                .wrap(middlewares::auth::Authentication)
                .route("/create", web::post().to(todo_controllers::todo_create))
                .route("/get", web::get().to(todo_controllers::todo_get))
                .route("/edit", web::post().to(todo_controllers::todo_edit))
                .route("/delete", web::post().to(todo_controllers::todo_delete))
                .route("/reset", web::post().to(todo_controllers::todo_reset)),
        )
        .service(
            web::scope("/api/account")
                .route("/register", web::post().to(account_controllers::account_register))
                .route("/login", web::post().to(account_controllers::account_login))
                .route("/reset", web::post().to(account_controllers::accounts_reset)),
        );
}

pub fn create_db_pool() -> Result<Pool, ConfigError> {
    let host = std::env::var("POSTGRES_HOST").expect("POSTGRES_HOST variable missing");
    let user = std::env::var("POSTGRES_USER").expect("POSTGRES_USER variable missing");
    let db = std::env::var("POSTGRES_DB").expect("POSTGRES_DB variable missing");
    let password = std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD variable missing");

    panic_after(Duration::from_secs(5), "DB timeout", move || {
        let default = Config::default();
        let cfg = Config {
            host: Some(host),
            user: Some(user),
            password: Some(password),
            dbname: Some(db),
            ..default
        };
        cfg.create_pool(NoTls)
    })
}

pub async fn create_redis_client() -> redis::RedisResult<redis::aio::Connection> {
    let host = std::env::var("REDIS_HOST").expect("REDIS_HOST variable missing");
    let port = std::env::var("REDIS_PORT").expect("REDIS_PORT variable missing");
    let mut client = redis::Client::open(format!("redis://{}:{}", host, port))?;
    if !client.check_connection() {
        panic!("Can't connect to redis");
    }

    let connection = client.get_async_connection().await;

    connection
}

pub fn get_session_id(response_headers: &HeaderMap) -> &str {
    let cookie_regex = Regex::new(r###"session_id=(.+?);"###).expect("Couldn't create email regex");
    let mut session_id = "";

    for (_, val) in response_headers.iter() {
        let cookie_str = val.to_str().expect("Can't parse cookie");
        let caps = cookie_regex.captures(cookie_str);
        if let Some(groups) = caps {
            session_id = groups.get(1).unwrap().as_str();
        }
    }

    session_id
}

#[allow(dead_code)]
pub async fn get_response_body<B>(response: ServiceResponse<B>) -> Value
where
    B: MessageBody,
{
    let response_body = test::read_body(response).await;
    serde_json::from_slice(response_body.as_ref()).expect("Can't parse to serde Value")
}

fn panic_after<T, F>(d: Duration, message: &'static str, f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T,
    F: Send + 'static,
{
    let (done_tx, done_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let val = f();
        done_tx.send(()).expect("Unable to send completion signal");
        val
    });

    match done_rx.recv_timeout(d) {
        Ok(_) => handle.join().expect("Thread panicked"),
        Err(_) => panic!(message),
    }
}
