use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json
};

use crate::repositories::quest::{QuestRepository, CreateQuest, UpdateQuest};

pub async fn create_quest<T: QuestRepository>(
    Json(payload): Json<CreateQuest>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest = repository
        .create(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(quest)))
}

pub async fn find_quest<T: QuestRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest = repository
        .find(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(quest)))
}

pub async fn all_quests<T: QuestRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quests = repository
        .all()
        .await
        .unwrap();

    Ok((StatusCode::OK, Json(quests)))
}

pub async fn update_quest<T: QuestRepository>(
    Path(id): Path<String>,
    Json(payload): Json<UpdateQuest>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest = repository
        .update(id, payload)
        .await
        .unwrap();

    Ok((StatusCode::OK, Json(quest)))
}

pub async fn delete_quest<T: QuestRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
}
