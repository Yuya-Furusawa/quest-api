use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json, TypedHeader,
};
use std::sync::Arc;

use crate::{
    repositories::{
        user_challenge::{CompleteChallengePayload, UserChallengeRepository},
        user_quest::UserQuestRepository,
    },
    services::user::decode_jwt,
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
    TypedHeader(cookie): TypedHeader<axum::headers::Cookie>,
    Extension(state): Extension<UserInfoHandlerState<T, S>>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(cookie_token) = cookie.get("session_token") {
        let secret_key = &state.secret_key;

        let decoded_token = decode_jwt(cookie_token, &secret_key).unwrap();

        let quest_ids = state
            .userchallenge_repository
            .get_completed_challenges_by_user_id(decoded_token.claims.user_id)
            .await
            .or(Err(StatusCode::NOT_FOUND))?;

        return Ok((StatusCode::OK, Json(quest_ids)));
    }

    Err(StatusCode::UNAUTHORIZED)
}
