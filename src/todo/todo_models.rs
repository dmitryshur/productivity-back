use chrono::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Todo {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
    creation_date: DateTime<Utc>,
    last_edit_date: DateTime<Utc>,
    done: bool,
}
