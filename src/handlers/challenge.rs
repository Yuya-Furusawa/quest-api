use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::challenge::{
    ChallengeRepository, CreateChallenge, FindChallengeByQuestId,
};

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

pub async fn find_challenge_by_quest_id<T: ChallengeRepository>(
    Query(payload): Query<FindChallengeByQuestId>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let challenges = repository
        .find_by_quest_id(payload.quest_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(challenges)))
}
