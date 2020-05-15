use crate::common::responses::ServerResponse;
use crate::todos::todo_models::{Todo, TodoDbExecutor};
use crate::AppState;
use actix_http::httpmessage::HttpMessage;
use actix_web::{self, dev, error, http, web, HttpRequest};
use chrono::prelude::*;
use postgres;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TodoCreateRequest {
    title: String,
    body: Option<String>,
}

#[derive(Deserialize)]
pub struct TodoGetRequest {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct TodoEditRequest {
    id: i32,
    title: Option<String>,
    body: Option<String>,
    done: Option<bool>,
}

#[derive(Deserialize)]
pub struct TodoDeleteRequest {
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
    Db(postgres::Error),
    Server,
}

impl std::fmt::Display for TodoErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for TodoErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let response_json = match self {
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
    let account_id = request.cookie("account_id").unwrap().value().parse::<i32>().unwrap();

    let current_date = Utc::now();
    let rows = TodoDbExecutor::create(
        &state.db_pool,
        &[&account_id, &body.title, &body.body, &current_date, &current_date],
    )
    .await;

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

            return Err(TodoErrors::Db(err));
        }
    }
}

pub async fn todo_get(
    request: HttpRequest,
    query: web::Query<TodoGetRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    let account_id = request.cookie("account_id").unwrap().value().parse::<i32>().unwrap();

    let rows = TodoDbExecutor::get(&state.db_pool, &[&account_id, &query.offset, &query.limit]).await;
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

            return Err(TodoErrors::Db(err));
        }
    }
}

pub async fn todo_edit(
    request: HttpRequest,
    body: web::Json<TodoEditRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    let account_id = request.cookie("account_id").unwrap().value().parse::<i32>().unwrap();
    let todo_id = body.id;
    let current_date = Utc::now();

    let rows = TodoDbExecutor::edit(
        &state.db_pool,
        &[
            &body.title,
            &body.body,
            &body.done,
            &current_date,
            &account_id,
            &body.id,
        ],
    )
    .await;

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

            return Err(TodoErrors::Db(err));
        }
    }
}

pub async fn todo_delete(
    request: HttpRequest,
    body: web::Json<TodoDeleteRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    let account_id = request.cookie("account_id").unwrap().value().parse::<i32>().unwrap();

    let rows = TodoDbExecutor::delete(&state.db_pool, &[&account_id, &body.todos]).await;
    match rows {
        Ok(rows) => {
            let todo_ids: Vec<i32> = rows.iter().map(|row| row.get("id")).collect();

            let data = TodoDeleteResponse { todos: todo_ids };
            let response_json = ServerResponse::new(data, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            return Err(TodoErrors::Db(err));
        }
    }
}

pub async fn todo_reset(state: web::Data<AppState>) -> actix_web::Result<actix_web::HttpResponse, TodoErrors> {
    let result = TodoDbExecutor::reset(&state.db_pool).await;
    match result {
        Ok(_) => Ok(actix_web::HttpResponse::Ok().finish()),
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            return Err(TodoErrors::Server);
        }
    }
}
