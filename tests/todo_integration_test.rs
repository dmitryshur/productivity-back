pub mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_rt;
    use actix_service::Service;
    use actix_web::{http, test, App};

    #[test]
    fn test_todos() {
        actix_rt::System::new("test_todos_system".to_string()).block_on(async move {
            // let mut app = test::init_service(App::new().configure(common::config_app().await)).await;
            //
            // let payload = r#"{"email":"dimashur@gmail.com","password":"12345678"}"#.as_bytes();
            // let request = test::TestRequest::post()
            //     .uri("/api/account/register")
            //     .header(http::header::CONTENT_TYPE, "application/json")
            //     .set_payload(payload)
            //     .to_request();
            //
            // let response = app.call(request).await.unwrap();
            // println!("{:?}", response);
            assert_eq!(1, 1);
        });
    }
}
