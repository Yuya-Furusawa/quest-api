use anyhow::Ok;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::challenge::Challenge;

#[async_trait]
pub trait QuestRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateQuest) -> anyhow::Result<QuestEntity>;
    async fn find(&self, id: String) -> anyhow::Result<QuestEntity>;
    async fn all(&self) -> anyhow::Result<Vec<QuestEntity>>;
    async fn update(&self, id: String, payload: UpdateQuest) -> anyhow::Result<QuestEntity>;
    async fn delete(&self, id: String) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct QuestRepositoryForDb {
    pool: PgPool,
}

impl QuestRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        QuestRepositoryForDb { pool }
    }

    #[cfg(test)]
    /// テスト用の簡易版コンストラクタ
    pub async fn with_url(url: &str) -> Self {
        let pool = PgPool::connect(url).await.unwrap();
        QuestRepositoryForDb::new(pool)
    }
}

#[async_trait]
impl QuestRepository for QuestRepositoryForDb {
    async fn create(&self, payload: CreateQuest) -> anyhow::Result<QuestEntity> {
        let row = sqlx::query_as::<_, QuestFromRow>(
            r#"
                insert into quests values ($1, $2, $3)
                returning *
            "#,
        )
        .bind(nanoid!())
        .bind(payload.title)
        .bind(payload.description)
        .fetch_one(&self.pool)
        .await?;

        let quest = QuestEntity::new(row.id, row.title, row.description);

        Ok(quest)
    }

    async fn find(&self, id: String) -> anyhow::Result<QuestEntity> {
        let row = sqlx::query_as::<_, QuestFromRow>(
            r#"
                select * from quests where id = $1;
            "#,
        )
        .bind(id.clone())
        .fetch_one(&self.pool)
        .await?;

        let challenges = sqlx::query_as::<_, Challenge>(
            r#"
                select * from challenges where quest_id = $1;
            "#,
        )
        .bind(id.clone())
        .fetch_all(&self.pool)
        .await?;

        let quest = QuestEntity {
            id: row.id,
            title: row.title,
            description: row.description,
            challenges,
        };

        Ok(quest)
    }

    async fn all(&self) -> anyhow::Result<Vec<QuestEntity>> {
        let quest_rows = sqlx::query_as::<_, QuestFromRow>(
            r#"
                select * from quests;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let challenge_rows = sqlx::query_as::<_, Challenge>(
            r#"
                select * from challenges;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut quests = quest_rows
            .into_iter()
            .map(|row| QuestEntity::new(row.id, row.title, row.description))
            .collect::<Vec<QuestEntity>>();

        for challenge in challenge_rows {
            if let Some(quest) = quests.iter_mut().find(|q| q.id == challenge.quest_id) {
                quest.challenges.push(challenge)
            }
        }

        Ok(quests)
    }

    async fn update(&self, id: String, payload: UpdateQuest) -> anyhow::Result<QuestEntity> {
        let old_quest = self.find(id.clone()).await?;
        let row = sqlx::query_as::<_, QuestFromRow>(
            r#"
                update quests set title=$1, description=$2 where id=$3
                returning *
            "#,
        )
        .bind(payload.title.unwrap_or(old_quest.title))
        .bind(payload.description.unwrap_or(old_quest.description))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let quest = QuestEntity {
            id: row.id,
            title: row.title,
            description: row.description,
            challenges: old_quest.challenges,
        };

        Ok(quest)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        sqlx::query(
            r#"
                delete from quests where id=$1
            "#,
        )
        .bind(id.clone())
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
                delete from challenges where quest_id=$1
            "#,
        )
        .bind(id.clone())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestFromRow {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestEntity {
    pub id: String,
    pub title: String,
    pub description: String,
    pub challenges: Vec<Challenge>,
}

impl QuestEntity {
    pub fn new(id: String, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description,
            challenges: Vec::new(),
        }
    }
}

// 各fieldが一致したとき==とみなす
impl PartialEq for QuestEntity {
    fn eq(&self, other: &QuestEntity) -> bool {
        (self.title == other.title) && (self.description == other.description)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuest {
    title: String,
    description: String,
}

#[cfg(test)]
impl CreateQuest {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuest {
    title: Option<String>,
    description: Option<String>,
}
