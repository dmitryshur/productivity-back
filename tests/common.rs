use actix_web::web;
use postgres;
use productivity::{account::account_controllers, middlewares, todos::todo_controllers};
use r2d2;
use r2d2_postgres::PostgresConnectionManager;
use redis;
use redis::ConnectionLike;
use std::{sync::mpsc, thread, time::Duration};

#[cfg(test)]
pub fn test_config_app(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api/todo")
                .wrap(middlewares::auth::Authentication)
                .route("/create", web::post().to(todo_controllers::todo_create))
                .route("/get", web::get().to(todo_controllers::todo_get))
                .route("/edit", web::post().to(todo_controllers::todo_edit))
                .route("/delete", web::post().to(todo_controllers::todo_delete)),
        )
        .service(
            web::scope("/api/account")
                .route("/register", web::post().to(account_controllers::account_register))
                .route("/login", web::post().to(account_controllers::account_login))
                .route("/reset", web::post().to(account_controllers::accounts_reset)),
        );
}

pub fn create_db_pool() -> Result<r2d2::Pool<PostgresConnectionManager<postgres::NoTls>>, r2d2::Error> {
    panic_after(Duration::from_secs(5), "DB timeout", || {
        let db_manager = PostgresConnectionManager::new(
            "host=localhost user=dshur dbname=productivity password=1234"
                .parse()
                .unwrap(),
            postgres::NoTls,
        );

        r2d2::Pool::new(db_manager)
    })
}

pub async fn create_redis_client() -> redis::RedisResult<redis::aio::Connection> {
    let mut client = redis::Client::open("redis://127.0.0.1:6379")?;
    if !client.check_connection() {
        panic!("Can't connect to redis");
    }

    let connection = client.get_async_connection().await;

    connection
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
