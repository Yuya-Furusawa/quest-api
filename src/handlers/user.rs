use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    headers::Cookie,
    http::{header, StatusCode},
    response::IntoResponse,
    Json, TypedHeader,
};

use crate::repositories::{
    quest::{QuestFromRow, QuestRepository},
    user::{LoginUser, ParticipateQuest, RegisterUser, UserRepository},
};
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

pub async fn participate_quest<T: UserRepository, U: QuestRepository>(
    Json(payload): Json<ParticipateQuest>,
    Extension(user_repository): Extension<Arc<T>>,
    Extension(quest_repository): Extension<Arc<U>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user_repository
        .find(payload.user_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;
    let quest = quest_repository
        .find(payload.quest_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    let quest_row = QuestFromRow {
        id: quest.id,
        title: quest.title,
        description: quest.description,
        price: quest.price,
        difficulty: quest.difficulty,
        num_participate: quest.num_participate,
        num_clear: quest.num_clear,
    };

    let updated_user = user_repository
        .participate_quest(user, quest_row)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(updated_user)))
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
