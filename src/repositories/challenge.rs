use anyhow::Ok;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[async_trait]
pub trait ChallengeRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateChallenge) -> anyhow::Result<Challenge>;
    async fn find(&self, id: String) -> anyhow::Result<Challenge>;
    async fn find_by_quest_id(&self, quest_id: String) -> anyhow::Result<Vec<Challenge>>;
}

#[derive(Debug, Clone)]
pub struct ChallengeRepositoryForDb {
    pool: PgPool,
}

impl ChallengeRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        ChallengeRepositoryForDb { pool }
    }
}

#[async_trait]
impl ChallengeRepository for ChallengeRepositoryForDb {
    async fn create(&self, payload: CreateChallenge) -> anyhow::Result<Challenge> {
        let challenge = sqlx::query_as::<_, Challenge>(
            r#"
				insert into challenges values ($1, $2, $3, $4, $5, $6)
				returning *
			"#,
        )
        .bind(nanoid!())
        .bind(payload.name)
        .bind(payload.description)
        .bind(payload.quest_id)
        .bind(payload.latitude)
        .bind(payload.longitude)
        .fetch_one(&self.pool)
        .await?;

        Ok(challenge)
    }

    async fn find(&self, id: String) -> anyhow::Result<Challenge> {
        let challenge = sqlx::query_as::<_, Challenge>(
            r#"
				select * from challenges where id = $1;
			"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(challenge)
    }

    async fn find_by_quest_id(&self, quest_id: String) -> anyhow::Result<Vec<Challenge>> {
        let challenges = sqlx::query_as::<_, Challenge>(
            r#"
                select * from challenges where quest_id = $1;
            "#,
        )
        .bind(quest_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(challenges)
    }
}

type ChallengeDatas = HashMap<String, Challenge>;

#[derive(Debug, Clone)]
pub struct ChallengeRepositoryForMemory {
    store: Arc<RwLock<ChallengeDatas>>,
}

impl ChallengeRepositoryForMemory {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<ChallengeDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<ChallengeDatas> {
        self.store.read().unwrap()
    }
}

#[async_trait]
impl ChallengeRepository for ChallengeRepositoryForMemory {
    async fn create(&self, payload: CreateChallenge) -> anyhow::Result<Challenge> {
        let mut store = self.write_store_ref();
        let id = nanoid!();
        let challenge = Challenge::new(
            id.clone(),
            payload.name,
            payload.description,
            payload.quest_id,
            payload.latitude,
            payload.longitude,
        );
        store.insert(id, challenge.clone());
        Ok(challenge)
    }

    async fn find(&self, id: String) -> anyhow::Result<Challenge> {
        let store = self.read_store_ref();
        let challenge = store.get(&id).map(|challenge| challenge.clone()).unwrap();
        Ok(challenge)
    }

    async fn find_by_quest_id(&self, quest_id: String) -> anyhow::Result<Vec<Challenge>> {
        let store = self.read_store_ref();
        let challenges = store
            .values()
            .filter(|challenge| challenge.quest_id == quest_id)
            .map(|challenge| challenge.clone())
            .collect::<Vec<Challenge>>();
        Ok(challenges)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Challenge {
    pub id: String,
    name: String,
    description: String,
    pub quest_id: String,
    latitude: f64,
    longitude: f64,
}

impl Challenge {
    pub fn new(
        id: String,
        name: String,
        description: String,
        quest_id: String,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Self {
            id,
            name,
            description,
            quest_id,
            latitude,
            longitude,
        }
    }
}

// 各fieldが一致したとき==とみなす
impl PartialEq for Challenge {
    fn eq(&self, other: &Challenge) -> bool {
        (self.name == other.name)
            && (self.description == other.description)
            && (self.quest_id == other.quest_id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateChallenge {
    name: String,
    description: String,
    quest_id: String,
    latitude: f64,
    longitude: f64,
}

#[cfg(test)]
impl CreateChallenge {
    pub fn new(
        name: String,
        description: String,
        quest_id: String,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Self {
            name,
            description,
            quest_id,
            latitude,
            longitude,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FindChallengeByQuestId {
    pub quest_id: String,
}
