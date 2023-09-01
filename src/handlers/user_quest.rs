use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{
    repositories::{
        user_challenge::UserChallengeRepository,
        user_quest::{ParticipateQuestPayload, UserQuestRepository},
    },
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
    Extension(user_id): Extension<String>,
    Extension(state): Extension<UserInfoHandlerState<T, S>>,
) -> Result<impl IntoResponse, StatusCode> {
    let quest_ids = state
        .userquest_repository
        .get_participated_quests_by_user_id(user_id)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(quest_ids)))
}
