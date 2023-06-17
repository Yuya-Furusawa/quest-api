use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::{header::SET_COOKIE, StatusCode},
    response::{IntoResponse, Response},
    Json, TypedHeader,
};
use chrono::{Duration, Utc};
use cookie::{time::OffsetDateTime, Cookie, Expiration};

use crate::{
    repositories::user::{LoginUser, RegisterUser, UserRepository},
    services::user::{create_jwt, decode_jwt},
};

pub async fn register_user<T: UserRepository>(
    Json(payload): Json<RegisterUser>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository
        .register(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now + Duration::hours(8)).timestamp();

    let token = create_jwt(&user.id, iat, &exp);
    let cookie = Cookie::build("session_token", &token)
        .path("/")
        .expires(Expiration::from(
            OffsetDateTime::from_unix_timestamp(exp).unwrap(),
        ))
        .secure(true)
        .http_only(true)
        .finish();

    Ok((
        StatusCode::CREATED,
        [(SET_COOKIE, cookie.to_string())],
        Json(user.clone()),
    ))
}

pub async fn login_user<T: UserRepository>(
    Json(payload): Json<LoginUser>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository
        .login(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now + Duration::hours(8)).timestamp();

    let token = create_jwt(&user.id, iat, &exp);
    let cookie = Cookie::build("session_token", &token)
        .path("/")
        .expires(Expiration::from(
            OffsetDateTime::from_unix_timestamp(exp).unwrap(),
        ))
        .secure(true)
        .http_only(true)
        .finish();

    Ok((
        StatusCode::CREATED,
        [(SET_COOKIE, cookie.to_string())],
        Json(user.clone()),
    ))
}

pub async fn find_user<T: UserRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository.find(id).await.or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn delete_user<T: UserRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
}

pub enum AuthError {
    NotFoundCookie,
    NotFoundUser,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::NotFoundCookie => {
                return (StatusCode::UNAUTHORIZED, "Not found cookie").into_response()
            }
            AuthError::NotFoundUser => {
                return (StatusCode::NOT_FOUND, "Not found a user").into_response()
            }
        };
    }
}

pub async fn auth_user<T: UserRepository>(
    TypedHeader(cookie): TypedHeader<axum::headers::Cookie>,
    Extension(user_repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, AuthError> {
    if let Some(cookie_token) = cookie.get("session_token") {
        let decoded_token = decode_jwt(cookie_token).unwrap();

        let user = user_repository
            .find(decoded_token.claims.user_id)
            .await
            .or(Err(AuthError::NotFoundUser))?;

        return Ok((StatusCode::CREATED, Json(user)));
    }

    return Err(AuthError::NotFoundCookie);
}
