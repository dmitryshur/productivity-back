use crate::actors::db_actor::{DbErrors, TodoCreateMessage};
use crate::AppState;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{
    self, error, http,
    web::{Data, Json},
    HttpResponse,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct TodoCreateRequest {
    user_id: i32,
    title: String,
    body: Option<String>,
}

#[derive(Debug)]
pub enum TodoCreateErrors {
    Db,
    Server,
}

impl std::fmt::Display for TodoCreateErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for TodoCreateErrors {
    fn status_code(&self) -> StatusCode {
        match *self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        println!("the error is: {:?}", self);
        let error_json = match self {
            TodoCreateErrors::Server => json!({"error": "Interval server error"}),
            TodoCreateErrors::Db => json!({"error": "DB error"}),
        };

        HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(error_json)
    }
}

#[derive(Serialize)]
pub struct TodoCreateResponse {
    id: i32,
    creation_date: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct TodoGetRequest {
    user_id: i32,
    offset: Option<i32>,
    limit: Option<i32>,
}

pub async fn todo_create(
    request: Json<TodoCreateRequest>,
    state: Data<AppState>,
) -> actix_web::Result<HttpResponse, TodoCreateErrors> {
    let request_json = request.into_inner().clone();
    let rows = state
        .db_actor
        .send(TodoCreateMessage::new(
            request_json.user_id,
            request_json.title,
            request_json.body,
        ))
        .await
        .map_err(|_| DbErrors::Runtime)
        .and_then(|res| res);

    match rows {
        Ok(rows) => {
            let row = &rows[0];
            let response_json = TodoCreateResponse {
                id: row.get("id"),
                creation_date: row.get("creation_date"),
            };

            Ok(HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(_) => Err(TodoCreateErrors::Db),
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
