use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    headers::Cookie,
    http::{header, StatusCode},
    response::IntoResponse,
    Json, TypedHeader,
};

use crate::repositories::user::{LoginUser, RegisterUser, UserRepository};
use crate::services::user::create_session;

pub async fn register_user<T: UserRepository>(
    Json(payload): Json<RegisterUser>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository
        .register(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;
    let session_token = create_session(&user).await;

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, session_token.cookie())],
        Json(user),
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
    let session_token = create_session(&user).await;

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, session_token.cookie())],
        Json(user),
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

pub async fn auth_user<T: UserRepository>(
    TypedHeader(cookie): TypedHeader<Cookie>,
    Extension(user_repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    // セッションidがあるときはJSON形式のuserを返す
    if let Some(session_id) = cookie.get("session_id") {
        let user = user_repository
            .find(session_id.to_string())
            .await
            .or(Err(StatusCode::NOT_FOUND))?;
        return Ok((StatusCode::OK, Json(user)));
    }
    // セッションidが無かったときはからの文字列を返す
    return Err(StatusCode::NOT_FOUND);
}
