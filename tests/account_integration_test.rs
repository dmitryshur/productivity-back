mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::http::StatusCode;
    use actix_rt;
    use actix_service::Service;
    use actix_web::{http, test, App};
    use productivity::AppState;
    use r2d2::{self, Pool};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_account_register() {
        let db_pool = common::create_db_pool().expect("Can't create db pool");
        let db_pool = Pool::clone(&db_pool);

        actix_rt::System::new("test_account_register_runtime".to_string()).block_on(async move {
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

            // Delete all existing data. USED IN TESTS ONLY
            let request = test::TestRequest::post().uri("/api/account/reset").to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            // Initial registration
            let payload = r#"{"email":"dimashur@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            // Registration with the same credentials
            let payload = r#"{"email":"dimashur@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CONFLICT);

            // Invalid email
            let payload = r#"{"email":"dimashur2","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            // Invalid password
            let payload = r#"{"email":"dimashur2@gmail.com","password":"1234"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }

    #[test]
    fn test_account_login() {
        let db_pool = common::create_db_pool().expect("Can't create db pool");
        let db_pool = Pool::clone(&db_pool);

        actix_rt::System::new("test_account_login_runtime".to_string()).block_on(async move {
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

            // Delete all existing data. USED IN TESTS ONLY
            let request = test::TestRequest::post().uri("/api/account/reset").to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            // Initial registration
            let payload = r#"{"email":"dimashur@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            // Invalid email
            let payload = r#"{"email":"dimashur","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

            // Invalid password
            let payload = r#"{"email":"dimashur@gmail.com","password":"1234"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

            // Wrong credentials provided
            let payload = r#"{"email":"dimashur1@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

            // Successful login
            let payload = r#"{"email":"dimashur@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/login")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();
            let response = app.call(request).await.unwrap();
            let session_id = common::get_session_id(&response.headers());
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(session_id.len() > 0, true);
        });
    }
}
