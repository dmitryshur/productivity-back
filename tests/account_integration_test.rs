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
    fn test_account() {
        let db_pool = common::create_db_pool().expect("Can't create db pool");
        let db_pool = Pool::clone(&db_pool);
        println!("{:?}", db_pool);

        actix_rt::System::new("test_account_runtime".to_string()).block_on(async move {
            let redis_client = common::create_redis_client()
                .await
                .expect("Can't create redis connection");
            let redis_client = Arc::new(Mutex::new(redis_client));

            let mut app = test::init_service(
                App::new()
                    .data(AppState { db_pool, redis_client })
                    .configure(common::config_app),
            )
            .await;

            let payload = r#"{"email":"dimashur20@gmail.com","password":"12345678"}"#.as_bytes();
            let request = test::TestRequest::post()
                .uri("/api/account/register")
                .header(http::header::CONTENT_TYPE, "application/json")
                .set_payload(payload)
                .to_request();

            let response = app.call(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        });
    }
}
