use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::user_challenge::{CompleteChallengePayload, UserChallengeRepository};

pub async fn complete_challenge<T: UserChallengeRepository>(
    Json(payload): Json<CompleteChallengePayload>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = repository
        .complete_challenge(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(row)))
}
