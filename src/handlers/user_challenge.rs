use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{
    repositories::{
        user_challenge::{CompleteChallengePayload, UserChallengeRepository},
        user_quest::UserQuestRepository,
    },
    UserInfoHandlerState,
};

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

pub async fn get_completed_challenges<T: UserQuestRepository, S: UserChallengeRepository>(
    Extension(user_id): Extension<String>,
    Extension(state): Extension<UserInfoHandlerState<T, S>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest_ids = state
        .userchallenge_repository
        .get_completed_challenges_by_user_id(user_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(quest_ids)))
}
