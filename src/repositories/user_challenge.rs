use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    marker::{Send, Sync},
    sync::{Arc, RwLock, RwLockWriteGuard},
};

#[async_trait]
pub trait UserChallengeRepository: Clone + Send + Sync + 'static {
    async fn save_challenge_complete_event(
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
    async fn save_challenge_complete_event(
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

type UserChallengeDatas = HashMap<i32, CompleteChallenge>;

#[derive(Debug, Clone)]
pub struct UserChallengeRepositoryForMemory {
    store: Arc<RwLock<UserChallengeDatas>>,
}

impl UserChallengeRepositoryForMemory {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<UserChallengeDatas> {
        self.store.write().unwrap()
    }
}

#[async_trait]
impl UserChallengeRepository for UserChallengeRepositoryForMemory {
    async fn save_challenge_complete_event(
        &self,
        payload: CompleteChallengePayload,
    ) -> anyhow::Result<CompleteChallenge> {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let complete_challenge = CompleteChallenge {
            id: id.clone(),
            user_id: payload.user_id,
            challenge_id: payload.challenge_id,
        };
        store.insert(id, complete_challenge.clone());
        anyhow::Ok(complete_challenge)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
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
