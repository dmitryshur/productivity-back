use crate::common::responses::ServerResponse;
use crate::AppState;
use actix_http;
use actix_service::{Service, Transform};
use actix_web::web::BytesMut;
use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest, ServiceResponse},
    error, http, Error, HttpMessage,
};
use futures::future::{ok, Ready};
use futures::Future;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use tokio::stream::StreamExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestBody {
    account_id: i32,
}

#[derive(Debug)]
pub enum AuthErrors {
    Forbidden,
}

impl std::fmt::Display for AuthErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for AuthErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            AuthErrors::Forbidden => http::StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let response_json = match self {
            AuthErrors::Forbidden => ServerResponse::new((), json!({"error": "Access forbidden"})),
        };

        HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(response_json)
    }
}

pub struct Authentication;

impl<S: 'static, B> Transform<S> for Authentication
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for AuthenticationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();

        Box::pin(async move {
            let state = req.app_data::<AppState>().unwrap();
            let session_cookie_value = req
                .cookie("session_id")
                .ok_or(AuthErrors::Forbidden)?
                .value()
                .to_string();

            // Get the body out of the request
            let mut body = BytesMut::new();
            let mut stream = req.take_payload();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk?);
            }
            let request_body = serde_json::from_slice::<RequestBody>(&body).map_err(|_e| AuthErrors::Forbidden)?;

            let redis_client = state.redis_client.clone();
            let session_id: String = redis_client
                .lock()
                .await
                .get(request_body.account_id)
                .await
                .map_err(|_e| AuthErrors::Forbidden)?;

            if session_id != session_cookie_value {
                return Err(AuthErrors::Forbidden)?;
            }

            // Put a payload back into the request. needs to be done because it was consumed earlier
            // by the stream
            let mut payload = actix_http::h1::Payload::empty();
            payload.unread_data(body.into());
            req.set_payload(payload.into());

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}
