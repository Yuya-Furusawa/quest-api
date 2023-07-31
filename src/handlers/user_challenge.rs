use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::user_challenge::{CompleteChallengePayload, UserChallengeRepository};

pub async fn complete_challenge<T: UserChallengeRepository>(
    Path(challenge_id): Path<String>,
    Json(payload): Json<CompleteChallengePayload>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    repository
        .save_challenge_complete_event(payload.user_id, challenge_id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::CREATED)
}
