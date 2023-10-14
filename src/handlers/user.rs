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
    UserHandlerState,
};

pub async fn register_user<T: UserRepository>(
    Json(payload): Json<RegisterUser>,
    Extension(state): Extension<UserHandlerState<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let secret_key = state.secret_key;

    let user = state
        .user_repository
        .register(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now + Duration::hours(8)).timestamp();

    let token = create_jwt(&user.id, iat, &exp, &secret_key);
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
    Extension(state): Extension<UserHandlerState<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let secret_key = state.secret_key;

    let user = state
        .user_repository
        .login(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now + Duration::hours(8)).timestamp();

    let token = create_jwt(&user.id, iat, &exp, &secret_key);
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
    Extension(state): Extension<UserHandlerState<T>>,
    Extension(user_id_from_token): Extension<String>,
) -> Result<impl IntoResponse, StatusCode> {
    if id != user_id_from_token {
        return Err(StatusCode::FORBIDDEN);
    }

    let user = state
        .user_repository
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn delete_user<T: UserRepository>(
    Path(id): Path<String>,
    Extension(state): Extension<UserHandlerState<T>>,
    Extension(user_id_from_token): Extension<String>,
) -> StatusCode {
    if id != user_id_from_token {
        return StatusCode::FORBIDDEN;
    }

    state
        .user_repository
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
                tracing::error!("Not found cookie");
                return StatusCode::UNAUTHORIZED.into_response();
            }
            AuthError::NotFoundUser => {
                tracing::error!("Not found user");
                return StatusCode::NOT_FOUND.into_response();
            }
        };
    }
}

pub async fn auth_user<T: UserRepository>(
    TypedHeader(cookie): TypedHeader<axum::headers::Cookie>,
    Extension(state): Extension<UserHandlerState<T>>,
) -> Result<impl IntoResponse, AuthError> {
    if let Some(cookie_token) = cookie.get("session_token") {
        let secret_key = &state.secret_key;

        let decoded_token = decode_jwt(cookie_token, &secret_key).unwrap();

        let user = state
            .user_repository
            .find(decoded_token.claims.user_id)
            .await
            .or(Err(AuthError::NotFoundUser))?;

        return Ok((StatusCode::CREATED, Json(user)));
    }

    return Err(AuthError::NotFoundCookie);
}
