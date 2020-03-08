#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use actix::prelude::*;
use actix_web::{middleware, web, App, HttpServer};

mod actors;
mod todo;

use actors::db_actor::DbActor;
use std::sync::Arc;
use todo::todo_controllers::todo_create;

#[derive(Debug)]
pub struct AppState {
    db_actor: Arc<Addr<DbActor>>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let db_actor = Arc::new(DbActor::new().start());

    std::env::set_var("RUST_LOG", "warnings=warn,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(AppState {
                db_actor: db_actor.clone(),
            })
            .service(web::scope("/api/todo").route("/create", web::post().to(todo_create)))
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
}
