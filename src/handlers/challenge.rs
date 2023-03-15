use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::challenge::{ChallengeRepository, CreateChallenge};

pub async fn create_challenge<T: ChallengeRepository>(
    Json(payload): Json<CreateChallenge>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let challenge = repository
        .create(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(challenge)))
}

pub async fn find_challenge<T: ChallengeRepository>(
    Path(id): Path<String>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let challenge = repository.find(id).await.or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(challenge)))
}
