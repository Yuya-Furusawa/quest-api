use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

#[async_trait]
pub trait UserQuestRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn participate_quest(
        &self,
        payload: ParticipateQuestPayload,
    ) -> anyhow::Result<ParticipateQuest>;
}

#[derive(Debug, Clone)]
pub struct UserQuestRepositoryForDb {
    pool: PgPool,
}

impl UserQuestRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        UserQuestRepositoryForDb { pool }
    }
}

#[async_trait]
impl UserQuestRepository for UserQuestRepositoryForDb {
    async fn participate_quest(
        &self,
        payload: ParticipateQuestPayload,
    ) -> anyhow::Result<ParticipateQuest> {
        let row = sqlx::query_as::<_, ParticipateQuest>(
            r#"
				insert into user_quests (user_id, quest_id) values ($1, $2)
				returning *
			"#,
        )
        .bind(payload.user_id)
        .bind(payload.quest_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(row)
    }
}

type UserQuestDatas = HashMap<i32, ParticipateQuest>;

#[derive(Debug, Clone)]
pub struct UserQuestRepositoryForMemory {
    store: Arc<RwLock<UserQuestDatas>>,
}

impl UserQuestRepositoryForMemory {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<UserQuestDatas> {
        self.store.write().unwrap()
    }
}

#[async_trait]
impl UserQuestRepository for UserQuestRepositoryForMemory {
    async fn participate_quest(
        &self,
        payload: ParticipateQuestPayload,
    ) -> anyhow::Result<ParticipateQuest> {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let participate_quest = ParticipateQuest {
            id: id.clone(),
            user_id: payload.user_id,
            quest_id: payload.quest_id,
        };
        store.insert(id, participate_quest.clone());
        anyhow::Ok(participate_quest)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
pub struct ParticipateQuest {
    pub id: i32,
    pub user_id: String,
    pub quest_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticipateQuestPayload {
    user_id: String,
    quest_id: String,
}
