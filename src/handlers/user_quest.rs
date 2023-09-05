use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json, TypedHeader,
};
use std::sync::Arc;

use crate::{
    repositories::{
        user_challenge::UserChallengeRepository,
        user_quest::{ParticipateQuestPayload, UserQuestRepository},
    },
    services::user::decode_jwt,
    UserInfoHandlerState,
};

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

pub async fn get_participated_quests<T: UserQuestRepository, S: UserChallengeRepository>(
    TypedHeader(cookie): TypedHeader<axum::headers::Cookie>,
    Extension(state): Extension<UserInfoHandlerState<T, S>>,
) -> Result<impl IntoResponse, StatusCode> {
    let cookie_token = match cookie.get("session_token") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(token) => token,
    };

    let secret_key = &state.secret_key;
    let decoded_token = decode_jwt(cookie_token, &secret_key).unwrap();

    let quest_ids = state
        .userquest_repository
        .get_participated_quests_by_user_id(decoded_token.claims.user_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(quest_ids)))
}
