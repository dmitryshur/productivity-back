use crate::todos::todo_models::TodoDbExecutor;
use crate::AppState;
use actix_web::{self, dev, error, http, web};
use chrono::prelude::*;
use postgres;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct TodoCreateRequest {
    user_id: i32,
    title: String,
    body: Option<String>,
}

#[derive(Serialize)]
pub struct TodoCreateResponse {
    id: i32,
    creation_date: DateTime<Utc>,
}

#[derive(Debug)]
pub enum TodoCreateErrors {
    Db(postgres::Error),
    Server,
}

impl std::fmt::Display for TodoCreateErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for TodoCreateErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let error_json = match self {
            TodoCreateErrors::Server => json!({"error": "Interval server error"}),
            TodoCreateErrors::Db(_e) => json!({"error": "DB error"}),
        };

        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(error_json)
    }
}

#[derive(Debug)]
pub enum DbErrors {
    Postgres(postgres::Error),
    Runtime,
}

pub async fn todo_create(
    request: web::Json<TodoCreateRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, TodoCreateErrors> {
    let pool = state.db_pool.clone();

    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        TodoDbExecutor::new(connection).create(&[&request.user_id, &request.title, &request.body])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            let row = &rows[0];
            let response_json = TodoCreateResponse {
                id: row.get("id"),
                creation_date: row.get("creation_date"),
            };

            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(e) => Err(TodoCreateErrors::Db(e)),
                DbErrors::Runtime => Err(TodoCreateErrors::Server),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test;
}
