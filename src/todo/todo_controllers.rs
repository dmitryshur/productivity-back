use crate::actors::db_actor::TodoCreateMessage;
use crate::AppState;
use actix_web::{
    web::{Data, Json},
    HttpRequest,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

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

#[derive(Deserialize, Debug)]
pub struct TodoGetRequest {
    user_id: i32,
    offset: Option<i32>,
    limit: Option<i32>,
}

pub async fn todo_create(
    request: Json<TodoCreateRequest>,
    state: Data<AppState>,
) -> Result<Json<TodoCreateResponse>, ()> {
    let request_json = request.into_inner().clone();
    // TODO if an error returns, map it to a custom error message
    let rows = state
        .db_actor
        .send(TodoCreateMessage::new(
            request_json.user_id,
            request_json.title,
            request_json.body,
        ))
        .await
        .map_err(|_| ())
        .and_then(|result| result);

    if let Ok(rows) = rows {
        let row = &rows[0];

        let response_json = TodoCreateResponse {
            id: row.get("id"),
            creation_date: row.get("creation_date"),
        };

        return Ok(Json(response_json));
    }

    Err(())
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test;
}
