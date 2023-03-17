use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::user_quest::{ParticipateQuestPayload, UserQuestRepository};

pub async fn participate_quest<T: UserQuestRepository>(
    Json(payload): Json<ParticipateQuestPayload>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = repository
        .participate_quest(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(row)))
}
