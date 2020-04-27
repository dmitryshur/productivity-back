pub mod common;

#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::cookie::Cookie;
    use actix_http::http::StatusCode;
    use actix_rt;
    use actix_service::Service;
    use actix_web::{http, test, App};
    use productivity::AppState;
    use r2d2::Pool;
    use serde_json::{self, Value};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_todos_create() {
        let db_pool = common::create_db_pool().expect("Can't create db pool");
        let db_pool = Pool::clone(&db_pool);

        actix_rt::System::new("test_todos_system".to_string()).block_on(async move {
            let redis_client = common::create_redis_client()
                .await
                .expect("Can't create redis connection");
            let redis_client = Arc::new(Mutex::new(redis_client));

            let mut app = test::init_service(
                App::new()
                    .data(AppState { db_pool, redis_client })
                    .configure(common::test_config_app),
            )
            .await;

            // Delete all existing account data. USED IN TESTS ONLY
            let request = test::TestRequest::post().uri("/api/account/reset").to_request();
            let response = test::call_service(&mut app, request).await;
            assert_eq!(response.status(), StatusCode::OK);

            // Initial registration
            let payload = json!({"email": "dimashur@gmail.com", "password": "12345678"});
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload.to_string())
                .to_request();
            let response = test::call_service(&mut app, request).await;
            assert_eq!(response.status(), StatusCode::OK);

            // Successful login
            let payload = json!({"email": "dimashur@gmail.com", "password": "12345678"});
            let request = test::TestRequest::post()
                .uri("/api/account/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload.to_string())
                .to_request();
            let response = test::call_service(&mut app, request).await;
            let session_id = common::get_session_id(&response.headers()).to_string();
            let response_body = test::read_body(response).await;
            let response_body: Value =
                serde_json::from_slice(response_body.as_ref()).expect("Can't parse to serde Value");
            let account_id = response_body["data"]["account_id"]
                .as_u64()
                .expect("Can't parse account_id");

            // Delete all existing data. USED IN TESTS ONLY
            let request = test::TestRequest::post()
                .uri("/api/todo/reset")
                .cookie(Cookie::new("session_id", session_id.clone()))
                .set_payload(json!({ "account_id": account_id }).to_string())
                .header(http::header::CONTENT_TYPE, "application/json")
                .to_request();
            let response = test::call_service(&mut app, request).await;
            assert_eq!(response.status(), StatusCode::OK);

            // Create todos without cookie
            let payload = json!({"title": "hello", "body": "world"});
            let request = test::TestRequest::post()
                .uri("/api/todo/create")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload.to_string())
                .to_request();
            let response = app.call(request).await;
            assert_eq!(response.is_err(), true);

            // Create todos with cookie
            let payload = json!({"account_id": account_id, "title": "hello", "body": "world"});
            let request = test::TestRequest::post()
                .uri("/api/todo/create")
                .cookie(Cookie::new("session_id", session_id.clone()))
                .set_payload(payload.to_string())
                .header(http::header::CONTENT_TYPE, "application/json")
                .to_request();
            let response = test::call_service(&mut app, request).await;
            assert_eq!(response.status(), StatusCode::OK);
        });
    }
}
