use crate::actors::db_actor::{DbErrors, TodoCreateMessage};
use crate::AppState;
use actix_web::{
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

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
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

pub async fn todo_create(request: Json<TodoCreateRequest>, state: Data<AppState>) -> HttpResponse {
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

            HttpResponse::Ok().json(response_json)
        }
        Err(e) => {
            let response_json = ErrorResponse {
                error: "DB error".to_owned(),
            };
            warn!(target: "warnings", "Warn: {:?}", e);

            HttpResponse::InternalServerError().json(response_json)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test;
}
