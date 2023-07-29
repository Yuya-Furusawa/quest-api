use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::user_challenge::{CompleteChallenge, UserChallengeRepository};

pub async fn complete_challenge<T: UserChallengeRepository>(
    Json(payload): Json<CompleteChallenge>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    repository
        .save_challenge_complete_event(payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::CREATED)
}
