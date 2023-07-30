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
        user_id: String,
        challenge_id: String,
    ) -> anyhow::Result<()>;
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
        user_id: String,
        challenge_id: String,
    ) -> anyhow::Result<()> {
        sqlx::query_as::<_, CompleteChallenge>(
            r#"
                insert into user_completed_challenges (user_id, challenge_id) values ($1, $2)
                returning *
            "#,
        )
        .bind(user_id)
        .bind(challenge_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(())
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

    #[cfg(test)]
    pub fn read_stored_value(&self) -> Vec<CompleteChallenge> {
        let store = self.store.read().unwrap();
        let challenges_vec = store
            .values()
            .map(|challenge| challenge.clone())
            .collect::<Vec<CompleteChallenge>>();

        challenges_vec
    }
}

#[async_trait]
impl UserChallengeRepository for UserChallengeRepositoryForMemory {
    async fn save_challenge_complete_event(
        &self,
        user_id: String,
        challenge_id: String,
    ) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        let id = store.len() as i32;
        store.insert(
            id,
            CompleteChallenge {
                user_id,
                challenge_id,
            },
        );

        anyhow::Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
pub struct CompleteChallenge {
    pub user_id: String,
    pub challenge_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompleteChallengePayload {
    pub user_id: String,
}
