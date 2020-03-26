use crate::account::account_models::DbErrors;
use crate::common::guards::Guard;
use crate::common::responses::ServerResponse;
use crate::todos::todo_models::{Todo, TodoDbExecutor};
use crate::AppState;
use actix_web::{self, dev, error, http, web, HttpRequest};
use chrono::prelude::*;
use postgres;
use redis::RedisError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TodoCreateRequest {
    account_id: i32,
    title: String,
    body: Option<String>,
}

#[derive(Deserialize)]
pub struct TodoGetRequest {
    account_id: i32,
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct TodoEditRequest {
    account_id: i32,
    id: i32,
    title: Option<String>,
    body: Option<String>,
    done: Option<bool>,
}

#[derive(Deserialize)]
pub struct TodoDeleteRequest {
    account_id: i32,
    todos: Vec<i32>,
}

#[derive(Serialize)]
pub struct TodoCreateResponse {
    id: i32,
    creation_date: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct TodoGetResponse {
    todos: Vec<Todo>,
}

#[derive(Serialize)]
pub struct TodoEditResponse {
    id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_edit_date: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct TodoDeleteResponse {
    todos: Vec<i32>,
}

#[derive(Debug)]
pub enum TodoErrors {
    Forbidden,
    Db(postgres::Error),
    Server,
}

impl From<RedisError> for TodoErrors {
    fn from(_: RedisError) -> Self {
        TodoErrors::Forbidden
    }
}

impl std::fmt::Display for TodoErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for TodoErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            TodoErrors::Forbidden => http::StatusCode::FORBIDDEN,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let response_json = match self {
            TodoErrors::Forbidden => ServerResponse::new((), json!({"error": "Access forbidden"})),
            TodoErrors::Server => ServerResponse::new((), json!({"error": "Interval server error"})),
            TodoErrors::Db(_e) => ServerResponse::new((), json!({"error": "DB error"})),
        };

        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(response_json)
    }
}

pub async fn todo_create(
    request: HttpRequest,
    body: web::Json<TodoCreateRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    Guard::auth(&request, &state, body.account_id).await?;

    let pool = state.db_pool.clone();
    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        let current_date = Utc::now();
        TodoDbExecutor::new(connection).create(&[
            &body.account_id,
            &body.title,
            &body.body,
            &current_date,
            &current_date,
        ])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            let row = &rows[0];
            let data = TodoCreateResponse {
                id: row.get("id"),
                creation_date: row.get("creation_date"),
            };

            let response_json = ServerResponse::new(data, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(e) => Err(TodoErrors::Db(e)),
                DbErrors::Runtime => Err(TodoErrors::Server),
            }
        }
    }
}

pub async fn todo_get(
    request: HttpRequest,
    body: web::Json<TodoGetRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    Guard::auth(&request, &state, body.account_id).await?;

    let pool = state.db_pool.clone();
    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        TodoDbExecutor::new(connection).get(&[&body.account_id, &body.offset, &body.limit])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            let todos: Vec<Todo> = rows
                .iter()
                .map(|row| {
                    let id = row.get("id");
                    let account_id = row.get("account_id");
                    let title = row.get("title");
                    let body = row.get("body");
                    let creation_date = row.get("creation_date");
                    let last_edit_date = row.get("last_edit_date");
                    let done = row.get("done");

                    Todo::new(id, account_id, title, body, creation_date, last_edit_date, done)
                })
                .collect();

            let data = TodoGetResponse { todos };
            let response_json = ServerResponse::new(data, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(e) => Err(TodoErrors::Db(e)),
                DbErrors::Runtime => Err(TodoErrors::Server),
            }
        }
    }
}

pub async fn todo_edit(
    request: HttpRequest,
    body: web::Json<TodoEditRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    Guard::auth(&request, &state, body.account_id).await?;

    let pool = state.db_pool.clone();
    let todo_id = body.id;

    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        let current_date = Utc::now();
        TodoDbExecutor::new(connection).edit(&[
            &body.title,
            &body.body,
            &body.done,
            &current_date,
            &body.account_id,
            &body.id,
        ])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            let data = if rows.is_empty() {
                TodoEditResponse {
                    id: todo_id,
                    last_edit_date: None,
                }
            } else {
                let row = &rows[0];
                TodoEditResponse {
                    id: row.get("id"),
                    last_edit_date: row.get("last_edit_date"),
                }
            };

            let response_json = ServerResponse::new(data, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(e) => Err(TodoErrors::Db(e)),
                DbErrors::Runtime => Err(TodoErrors::Server),
            }
        }
    }
}

pub async fn todo_delete(
    request: HttpRequest,
    body: web::Json<TodoDeleteRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    Guard::auth(&request, &state, body.account_id).await?;

    let pool = state.db_pool.clone();
    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        TodoDbExecutor::new(connection).delete(&[&body.account_id, &body.todos])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            let todo_ids: Vec<i32> = rows.iter().map(|row| row.get("id")).collect();

            let data = TodoDeleteResponse { todos: todo_ids };
            let response_json = ServerResponse::new(data, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(e) => Err(TodoErrors::Db(e)),
                DbErrors::Runtime => Err(TodoErrors::Server),
            }
        }
    }
}
