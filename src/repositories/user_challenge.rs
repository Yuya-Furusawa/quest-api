use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::marker::{Send, Sync};

#[async_trait]
pub trait UserChallengeRepository: Clone + Send + Sync + 'static {
    async fn complete_challnge(
        &self,
        payload: CompleteChallengePayload,
    ) -> anyhow::Result<CompleteChallenge>;
}

#[derive(Debug, Clone)]
pub struct UserChallengeRepositoryForDb {
    pool: PgPool,
}

impl UserChallengeRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        UserChallengeRepositoryForDb { pool }
    }
}

#[async_trait]
impl UserChallengeRepository for UserChallengeRepositoryForDb {
    async fn complete_challnge(
        &self,
        payload: CompleteChallengePayload,
    ) -> anyhow::Result<CompleteChallenge> {
        let row = sqlx::query_as::<_, CompleteChallenge>(
            r#"
				insert into user_challenges (user_id, challenge_id) values ($1, $2)
				returning *
			"#,
        )
        .bind(payload.user_id)
        .bind(payload.challenge_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(row)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct CompleteChallenge {
    pub id: i32,
    pub user_id: String,
    pub challenge_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompleteChallengePayload {
    user_id: String,
    challenge_id: String,
}
