use actix::prelude::*;
use actix_web::web;
use chrono::prelude::*;
use postgres::{Client, NoTls, Row};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

pub enum DbErrors {
    /// The specified user_id doesn't exist in the database
    NoUser,
    /// General database error
    Unknown,
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
#[rtype(result = "Result<Vec<Row>, ()>")]
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
    type Result = ResponseActFuture<Self, Result<Vec<Row>, ()>>;

    fn handle(&mut self, msg: TodoCreateMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut pool = self.pool.as_ref().unwrap().clone();
        let mut future = async {
            web::block(move || {
                let mut connection = pool.get().unwrap();

                // TODO save this db error and return it to the controller
                let rows = connection
                    .query(
                        "INSERT INTO todo(user_id, title, body) VALUES($1, $2, $3) RETURNING id, creation_date",
                        &[&msg.user_id, &msg.title, &msg.body],
                    )
                    .unwrap();
                Ok::<Vec<Row>, ()>(rows)
            })
            .await
            .map_err(|_| ())
        };

        Box::new(future.into_actor(self))
    }
}
