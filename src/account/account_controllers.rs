use crate::account::account_models::AccountDbExecutor;
use crate::common::responses::ServerResponse;
use crate::common::validators::{ValidationErrors, Validator};
use crate::AppState;
use actix_web::{self, cookie, dev, error, http, web};
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid;

const MONTH_IN_SECONDS: i64 = 2628000;

#[derive(Deserialize)]
pub struct AccountRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AccountLoginResponse {
    account_id: i32,
}

#[derive(Debug)]
pub enum AccountRegistrationErrors {
    InvalidEmail,
    InvalidPassword,
    EmailExists,
    Server,
    Db,
}

impl From<ValidationErrors> for AccountRegistrationErrors {
    fn from(err: ValidationErrors) -> AccountRegistrationErrors {
        match err {
            ValidationErrors::Email => AccountRegistrationErrors::InvalidEmail,
            ValidationErrors::Password => AccountRegistrationErrors::InvalidPassword,
        }
    }
}

impl std::fmt::Display for AccountRegistrationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for AccountRegistrationErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            AccountRegistrationErrors::InvalidEmail => http::StatusCode::UNPROCESSABLE_ENTITY,
            AccountRegistrationErrors::InvalidPassword => http::StatusCode::UNPROCESSABLE_ENTITY,
            AccountRegistrationErrors::EmailExists => http::StatusCode::CONFLICT,
            AccountRegistrationErrors::Server => http::StatusCode::INTERNAL_SERVER_ERROR,
            AccountRegistrationErrors::Db => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let response_json = match self {
            AccountRegistrationErrors::InvalidEmail => ServerResponse::new((), json!({"error": "Invalid email"})),
            AccountRegistrationErrors::InvalidPassword => ServerResponse::new(
                (),
                json!({"error": "Invalid password. the password must be at least 8 characters long"}),
            ),
            AccountRegistrationErrors::EmailExists => {
                ServerResponse::new((), json!({"error": "Such an email already exists"}))
            }
            AccountRegistrationErrors::Server => ServerResponse::new((), json!({"error": "Server error"})),
            AccountRegistrationErrors::Db => ServerResponse::new((), json!({"error": "Db error"})),
        };

        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(response_json)
    }
}

#[derive(Debug)]
pub enum AccountLoginErrors {
    InvalidInfo,
    Server,
}

impl From<ValidationErrors> for AccountLoginErrors {
    fn from(err: ValidationErrors) -> AccountLoginErrors {
        match err {
            _ => AccountLoginErrors::InvalidInfo,
        }
    }
}

impl From<error::Error> for AccountLoginErrors {
    fn from(err: error::Error) -> AccountLoginErrors {
        match err {
            _ => AccountLoginErrors::Server,
        }
    }
}

impl From<RedisError> for AccountLoginErrors {
    fn from(err: RedisError) -> AccountLoginErrors {
        match err {
            _ => AccountLoginErrors::Server,
        }
    }
}

impl std::fmt::Display for AccountLoginErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::ResponseError for AccountLoginErrors {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            AccountLoginErrors::InvalidInfo => http::StatusCode::UNAUTHORIZED,
            AccountLoginErrors::Server => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let response_json = match self {
            AccountLoginErrors::InvalidInfo => ServerResponse::new((), json!({"error": "Wrong email or password"})),
            AccountLoginErrors::Server => ServerResponse::new((), json!({"error": "Server error"})),
        };

        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(response_json)
    }
}

pub async fn account_register(
    body: web::Json<AccountRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountRegistrationErrors> {
    Validator::email(&body.email)?;
    Validator::password(&body.password)?;

    let rows_count = AccountDbExecutor::register(&state.db_pool, &[&body.email, &body.password]).await;

    match rows_count {
        Ok(_count) => {
            let response_json = ServerResponse::new((), ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err.code() {
                None => Err(AccountRegistrationErrors::Server),
                Some(err) => match err.code() {
                    "23505" => Err(AccountRegistrationErrors::EmailExists),
                    _ => Err(AccountRegistrationErrors::Db),
                },
            }
        }
    }
}

pub async fn account_login(
    body: web::Json<AccountRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountLoginErrors> {
    Validator::email(&body.email)?;
    Validator::password(&body.password)?;

    let rows = AccountDbExecutor::login(&state.db_pool, &[&body.email, &body.password]).await;
    match rows {
        Ok(rows) => {
            if rows.is_empty() {
                return Err(AccountLoginErrors::InvalidInfo);
            }
            let row = &rows[0];
            let account_id: i32 = row.get("id");
            let session_id = uuid::Uuid::new_v4().to_string();
            let _: () = state.redis_client.lock().await.set(account_id, &session_id).await?;

            let response_cookie = http::CookieBuilder::new("session_id", session_id)
                .max_age(MONTH_IN_SECONDS)
                .secure(false)
                .same_site(cookie::SameSite::Strict)
                .http_only(true)
                .finish();

            let response_json = ServerResponse::new(AccountLoginResponse { account_id }, ());
            Ok(actix_web::HttpResponse::Ok()
                .cookie(response_cookie)
                .json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            return Err(AccountLoginErrors::Server);
        }
    }
}

pub async fn accounts_reset(
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountLoginErrors> {
    let result = AccountDbExecutor::reset(&state.db_pool).await;

    match result {
        Ok(_) => Ok(actix_web::HttpResponse::Ok().finish()),
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            return Err(AccountLoginErrors::Server);
        }
    }
}
