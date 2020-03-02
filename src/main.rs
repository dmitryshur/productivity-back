use actix_web::{web, App, HttpServer};

mod todo;

use todo::todo_controllers::{todo_add, todo_delete, todo_edit, todo_get};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/api/todo")
                .route("/get", web::get().to(todo_get))
                .route("/add", web::post().to(todo_add))
                .route("/edit", web::post().to(todo_edit))
                .route("/delete", web::post().to(todo_delete)),
        )
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
}
