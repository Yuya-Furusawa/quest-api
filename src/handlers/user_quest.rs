use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::user_quest::{ParticipateQuest, UserQuestRepository};

pub async fn participate_quest<T: UserQuestRepository>(
    Json(payload): Json<ParticipateQuest>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    repository
        .save_quest_participate_event(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::CREATED)
}
