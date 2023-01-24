use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::repositories::{
    quest::QuestRepository,
    user::{LoginUser, ParticipateQuest, RegisterUser, UserRepository},
};

pub async fn register_user<T: UserRepository>(
    Json(payload): Json<RegisterUser>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository
        .register(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login_user<T: UserRepository>(
    Json(payload): Json<LoginUser>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = repository
        .login(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(user)))
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

    println!("{:?}", user);
    println!("{:?}", quest);

    let updated_user = user_repository
        .participate_quest(user, quest)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(updated_user)))
}
