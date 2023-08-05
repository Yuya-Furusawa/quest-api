use anyhow::Ok;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

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
                insert into quests values ($1, $2, $3, $4, $5, $6, $7)
                returning *
            "#,
        )
        .bind(nanoid!())
        .bind(payload.title)
        .bind(payload.description)
        .bind(payload.price)
        .bind(payload.difficulty.to_string())
        .bind(payload.num_participate)
        .bind(payload.num_clear)
        .fetch_one(&self.pool)
        .await?;

        let quest = QuestEntity::new(
            row.id,
            row.title,
            row.description,
            row.price,
            row.difficulty,
            row.num_participate,
            row.num_clear,
        );

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
            price: row.price,
            difficulty: row.difficulty,
            num_participate: row.num_participate,
            num_clear: row.num_clear,
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
            .map(|row| {
                QuestEntity::new(
                    row.id,
                    row.title,
                    row.description,
                    row.price,
                    row.difficulty,
                    row.num_participate,
                    row.num_clear,
                )
            })
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
                update quests set title=$1, description=$2, price=$3, difficulty=$4, num_participate=$5, num_clear=$6
                where id=$7
                returning *
            "#,
        )
        .bind(payload.title.unwrap_or(old_quest.title))
        .bind(payload.description.unwrap_or(old_quest.description))
        .bind(payload.price.unwrap_or(old_quest.price))
        .bind((payload.difficulty.unwrap_or(old_quest.difficulty)).to_string())
        .bind(payload.num_participate.unwrap_or(old_quest.num_participate))
        .bind(payload.num_clear.unwrap_or(old_quest.num_clear))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let quest = QuestEntity {
            id: row.id,
            title: row.title,
            description: row.description,
            price: row.price,
            difficulty: row.difficulty,
            num_participate: row.num_participate,
            num_clear: row.num_clear,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<String> for Difficulty {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let from_row: &str = &value;

        const EASY: &str = "Easy";
        const NORMAL: &str = "Normal";
        const HARD: &str = "Hard";

        match from_row {
            EASY => core::result::Result::Ok(Difficulty::Easy),
            NORMAL => core::result::Result::Ok(Difficulty::Normal),
            HARD => core::result::Result::Ok(Difficulty::Hard),
            _ => Err("Wrong Column Name"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestFromRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: i32, // 0ならFree
    #[sqlx(try_from = "String")]
    pub difficulty: Difficulty,
    pub num_participate: i32,
    pub num_clear: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestEntity {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: i32, // 0ならFree
    #[sqlx(try_from = "String")]
    pub difficulty: Difficulty,
    pub num_participate: i32,
    pub num_clear: i32,
    pub challenges: Vec<Challenge>,
}

impl QuestEntity {
    pub fn new(
        id: String,
        title: String,
        description: String,
        price: i32,
        difficulty: Difficulty,
        num_participate: i32,
        num_clear: i32,
    ) -> Self {
        Self {
            id,
            title,
            description,
            price,
            difficulty,
            num_participate,
            num_clear,
            challenges: Vec::new(),
        }
    }
}

// 各fieldが一致したとき==とみなす
impl PartialEq for QuestEntity {
    fn eq(&self, other: &QuestEntity) -> bool {
        (self.title == other.title)
            && (self.description == other.description)
            && (self.price == other.price)
            && (self.difficulty == other.difficulty)
            && (self.num_participate == other.num_participate)
            && (self.num_clear == other.num_clear)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuest {
    title: String,
    description: String,
    price: i32, // 0ならFree
    difficulty: Difficulty,
    num_participate: i32,
    num_clear: i32,
}

#[cfg(test)]
impl CreateQuest {
    pub fn new(
        title: String,
        description: String,
        price: i32,
        difficulty: Difficulty,
        num_participate: i32,
        num_clear: i32,
    ) -> Self {
        Self {
            title,
            description,
            price,
            difficulty,
            num_participate,
            num_clear,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuest {
    title: Option<String>,
    description: Option<String>,
    price: Option<i32>,
    difficulty: Option<Difficulty>,
    num_participate: Option<i32>,
    num_clear: Option<i32>,
}
