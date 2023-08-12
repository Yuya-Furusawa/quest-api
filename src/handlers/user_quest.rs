use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::user_quest::{ParticipateQuestPayload, UserQuestRepository};

pub async fn participate_quest<T: UserQuestRepository>(
    Path(quest_id): Path<String>,
    Json(payload): Json<ParticipateQuestPayload>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    repository
        .save_quest_participate_event(payload.user_id, quest_id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::CREATED)
}

pub async fn get_participated_quests<T: UserQuestRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest_ids = repository
        .get_participated_quests_by_user_id(id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(quest_ids)))
}
