use crate::todos::todo_controllers::TodoErrors;
use crate::AppState;
use actix_web::{web, HttpMessage, HttpRequest};
use redis::AsyncCommands;

pub struct Guard;

impl Guard {
    pub async fn auth(
        request: &HttpRequest,
        app_state: &web::Data<AppState>,
        account_id: i32,
    ) -> Result<(), TodoErrors> {
        let session_cookie_value = request
            .cookie("session_id")
            .ok_or(TodoErrors::Forbidden)?
            .value()
            .to_string();

        let redis_client = app_state.redis_client.clone();
        let session_id: String = redis_client.lock().await.get(account_id).await?;

        if session_cookie_value == session_id {
            return Ok(());
        }

        Err(TodoErrors::Forbidden)
    }
}
