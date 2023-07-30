use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

#[async_trait]
pub trait UserQuestRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn save_quest_participate_event(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()>;
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
    async fn save_quest_participate_event(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()> {
        sqlx::query_as::<_, ParticipateQuest>(
            r#"
				insert into user_participating_quests (user_id, quest_id) values ($1, $2)
				returning *
			"#,
        )
        .bind(user_id)
        .bind(quest_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(())
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

    #[cfg(test)]
    pub fn read_stored_value(&self) -> Vec<ParticipateQuest> {
        let store = self.store.read().unwrap();
        let quests_vec = store
            .values()
            .map(|challenge| challenge.clone())
            .collect::<Vec<ParticipateQuest>>();

        quests_vec
    }
}

#[async_trait]
impl UserQuestRepository for UserQuestRepositoryForMemory {
    async fn save_quest_participate_event(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        store.insert(id, ParticipateQuest { user_id, quest_id });

        anyhow::Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
pub struct ParticipateQuest {
    pub user_id: String,
    pub quest_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticipateQuestPayload {
    pub user_id: String,
}
