use actix::prelude::*;
use actix_web::error::BlockingError;
use actix_web::web;
use chrono::prelude::*;
use postgres::{self, NoTls, Row};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

#[derive(Debug)]
pub enum DbErrors {
    Postgres(postgres::Error),
    Runtime,
}

#[derive(Serialize, Debug)]
pub struct TodoModel {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
    creation_date: DateTime<Utc>,
    last_edit_date: DateTime<Utc>,
    done: bool,
}

#[derive(Debug)]
pub struct DbActor {
    pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
}

impl DbActor {
    pub fn new() -> Self {
        DbActor { pool: None }
    }
}

impl Actor for DbActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let manager = PostgresConnectionManager::new(
            "host=localhost user=dshur dbname=productivity password=1234"
                .parse()
                .unwrap(),
            NoTls,
        );
        let pool = Pool::new(manager).unwrap();

        self.pool = Some(pool.clone());
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Row>, DbErrors>")]
pub struct TodoCreateMessage {
    user_id: i32,
    title: String,
    body: Option<String>,
}

impl TodoCreateMessage {
    pub fn new(user_id: i32, title: String, body: Option<String>) -> Self {
        TodoCreateMessage { user_id, title, body }
    }
}

impl Handler<TodoCreateMessage> for DbActor {
    type Result = ResponseActFuture<Self, Result<Vec<Row>, DbErrors>>;

    fn handle(&mut self, msg: TodoCreateMessage, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.as_ref().unwrap().clone();
        let future = async {
            web::block(move || {
                let mut connection = pool.get().unwrap();

                connection.query(
                    "INSERT INTO todos(user_id, title, body) VALUES($1, $2, $3) RETURNING id, creation_date",
                    &[&msg.user_id, &msg.title, &msg.body],
                )
            })
            .await
            .map_err(|e| match e {
                BlockingError::Error(e) => DbErrors::Postgres(e),
                BlockingError::Canceled => DbErrors::Runtime,
            })
        };

        Box::new(future.into_actor(self))
    }
}
