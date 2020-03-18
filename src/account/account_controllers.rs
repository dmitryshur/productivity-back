use crate::account::account_models::{AccountDbExecutor, DbErrors};
use crate::AppState;
use actix_web::{self, dev, error, http, web};
use serde::{Deserialize};

#[derive(Deserialize)]
pub struct AccountRegisterRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct AccountLoginRequest {}

#[derive(Debug)]
pub enum AccountRegistrationErrors {
    InvalidEmail,
    InvalidPassword,
    EmailExists,
    Server,
    Db,
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
        let error_json = match self {
            AccountRegistrationErrors::InvalidEmail => json!({"error": "Invalid email"}),
            AccountRegistrationErrors::InvalidPassword => {
                json!({"error": "Invalid password. the password must be at least 8 characters long"})
            }
            AccountRegistrationErrors::EmailExists => json!({"error": "Such an email already exists"}),
            AccountRegistrationErrors::Server => json!({"error": "Server error"}),
            AccountRegistrationErrors::Db => json!({"error": "Db error"}),
        };

        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(error_json)
    }
}

#[derive(Debug)]
pub enum AccountLoginErrors {
    InvalidInfo,
    ServerError,
}

impl std::fmt::Display for AccountLoginErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub async fn account_register(
    request: web::Json<AccountRegisterRequest>,
    state: web::Data<AppState>,
) -> actix_web::Result<actix_web::HttpResponse, AccountRegistrationErrors> {
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
        Ok(_count) => Ok(actix_web::HttpResponse::Ok().json({})),
        Err(err) => {
            warn!(target: "warnings", "Warn: {:?}", err);

            match err {
                DbErrors::Postgres(_e) => Err(AccountRegistrationErrors::Db),
                DbErrors::Runtime => Err(AccountRegistrationErrors::Server),
            }
        }
    }
}
