use actix_web::{web::Json, HttpRequest};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Todo {
    title: String,
}

pub async fn todo_get(_request: HttpRequest) -> Result<Json<Todo>, ()> {
    let result = Todo {
        title: "get".to_owned(),
    };

    Ok(Json(result))
}

pub async fn todo_add(_request: HttpRequest) -> Result<Json<Todo>, ()> {
    let result = Todo {
        title: "add".to_owned(),
    };

    Ok(Json(result))
}

pub async fn todo_edit(_request: HttpRequest) -> Result<Json<Todo>, ()> {
    let result = Todo {
        title: "edit".to_owned(),
    };

    Ok(Json(result))
}

pub async fn todo_delete(_request: HttpRequest) -> Result<Json<Todo>, ()> {
    let result = Todo {
        title: "delete".to_owned(),
    };

    Ok(Json(result))
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_todo_get() {
        let request = test::TestRequest::get().to_http_request();
        let response = todo_get(request).await.unwrap();
        assert_eq!(response.title, "get".to_owned());
    }

    #[actix_rt::test]
    async fn test_todo_add() {
        let request = test::TestRequest::post().to_http_request();
        let response = todo_add(request).await.unwrap();
        assert_eq!(response.title, "add".to_owned());
    }

    #[actix_rt::test]
    async fn test_todo_edit() {
        let request = test::TestRequest::get().to_http_request();
        let response = todo_edit(request).await.unwrap();
        assert_eq!(response.title, "edit".to_owned());
    }

    #[actix_rt::test]
    async fn test_todo_delete() {
        let request = test::TestRequest::get().to_http_request();
        let response = todo_delete(request).await.unwrap();
        assert_eq!(response.title, "delete".to_owned());
    }
}
