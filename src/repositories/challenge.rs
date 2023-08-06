use anyhow::Ok;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

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

    #[cfg(test)]
    /// テスト用の簡易版コンストラクタ
    pub async fn with_url(url: &str) -> Self {
        let pool = PgPool::connect(url).await.unwrap();
        ChallengeRepositoryForDb::new(pool)
    }
}

#[async_trait]
impl ChallengeRepository for ChallengeRepositoryForDb {
    async fn create(&self, payload: CreateChallenge) -> anyhow::Result<Challenge> {
        let challenge = sqlx::query_as::<_, Challenge>(
            r#"
				insert into challenges values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
				returning *
			"#,
        )
        .bind(nanoid!())
        .bind(payload.name)
        .bind(payload.description)
        .bind(payload.quest_id)
        .bind(payload.latitude)
        .bind(payload.longitude)
        .bind(payload.stamp_name)
        .bind(payload.stamp_image_color)
        .bind(payload.stamp_image_gray)
        .bind(payload.flavor_text)
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

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Challenge {
    pub id: String,
    name: String,
    description: String,
    pub quest_id: String,
    latitude: f64,
    longitude: f64,
    stamp_name: String,
    stamp_image_color: String,
    stamp_image_gray: String,
    flavor_text: String,
}

impl Challenge {
    #[cfg(test)]
    pub fn new(
        id: String,
        name: String,
        description: String,
        quest_id: String,
        latitude: f64,
        longitude: f64,
        stamp_name: String,
        stamp_image_color: String,
        stamp_image_gray: String,
        flavor_text: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            quest_id,
            latitude,
            longitude,
            stamp_name,
            stamp_image_color,
            stamp_image_gray,
            flavor_text,
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
    stamp_name: String,
    stamp_image_color: String,
    stamp_image_gray: String,
    flavor_text: String,
}

#[cfg(test)]
impl CreateChallenge {
    pub fn new(
        name: String,
        description: String,
        quest_id: String,
        latitude: f64,
        longitude: f64,
        stamp_name: String,
        stamp_image_color: String,
        stamp_image_gray: String,
        flavor_text: String,
    ) -> Self {
        Self {
            name,
            description,
            quest_id,
            latitude,
            longitude,
            stamp_name,
            stamp_image_color,
            stamp_image_gray,
            flavor_text,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FindChallengeByQuestId {
    pub quest_id: String,
}
