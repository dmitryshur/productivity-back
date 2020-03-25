use crate::account::account_models::{AccountDbExecutor, DbErrors};
use crate::common::responses::ServerResponse;
use crate::common::validators::{ValidationErrors, Validator};
use crate::AppState;
use redis::AsyncCommands;
use actix_web::{self, dev, error, http, web};
use serde::{Deserialize, Serialize};

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
    request: web::Json<AccountRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountRegistrationErrors> {
    Validator::email(&request.email)?;
    Validator::password(&request.password)?;

    let pool = state.db_pool.clone();
    let rows_count = web::block(move || {
        let connection = pool.get().unwrap();
        AccountDbExecutor::new(connection).register(&[&request.email, &request.password])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows_count {
        Ok(_count) => {
            let response_json = ServerResponse::new((), ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Runtime => Err(AccountRegistrationErrors::Server),
                DbErrors::Postgres(e) => match e.code().unwrap().code() {
                    "23505" => Err(AccountRegistrationErrors::EmailExists),
                    _ => Err(AccountRegistrationErrors::Db),
                },
            }
        }
    }
}

pub async fn account_login(
    request: web::Json<AccountRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountLoginErrors> {
    Validator::email(&request.email)?;
    Validator::password(&request.password)?;

    let mut redis_connection = state
        .redis_client
        .get_async_connection()
        .await
        .expect("Can't get redis connection");

    let _ : () = redis_connection.set("key1", 42).await.expect("Can't set key");
    let pool = state.db_pool.clone();
    let rows = web::block(move || {
        let connection = pool.get().unwrap();
        AccountDbExecutor::new(connection).login(&[&request.email, &request.password])
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(e) => DbErrors::Postgres(e),
        error::BlockingError::Canceled => DbErrors::Runtime,
    });

    match rows {
        Ok(rows) => {
            if rows.is_empty() {
                return Err(AccountLoginErrors::InvalidInfo);
            }
            let row = &rows[0];
            let account_id: i32 = row.get("id");

            let response_json = ServerResponse::new(AccountLoginResponse { account_id }, ());
            Ok(actix_web::HttpResponse::Ok().json(response_json))
        }
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            return match err {
                DbErrors::Runtime => Err(AccountLoginErrors::Server),
                DbErrors::Postgres(_err) => Err(AccountLoginErrors::Server),
            };
        }
    }
}
