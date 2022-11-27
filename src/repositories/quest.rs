use anyhow::{Context, Ok};
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use std::{
    collections::HashMap,
    io::ErrorKind::NotFound,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[async_trait]
pub trait QuestRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateQuest) -> anyhow::Result<Quest>;
    async fn find(&self, id: String) -> anyhow::Result<Quest>;
    async fn all(&self) -> anyhow::Result<Vec<Quest>>;
    async fn update(&self, id: String, payload: UpdateQuest) -> anyhow::Result<Quest>;
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
}

#[async_trait]
impl QuestRepository for QuestRepositoryForDb {
    async fn create(&self, payload: CreateQuest) -> anyhow::Result<Quest> {
        let quest = sqlx::query_as::<_, Quest>(
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

        Ok(quest)
    }

    async fn find(&self, id: String) -> anyhow::Result<Quest> {
        let quest = sqlx::query_as::<_, Quest>(
            r#"
                select * from quests where id = $1;
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(quest)
    }

    async fn all(&self) -> anyhow::Result<Vec<Quest>> {
        let quests = sqlx::query_as::<_, Quest>(
            r#"
                select * from quests;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(quests)
    }

    async fn update(&self, id: String, payload: UpdateQuest) -> anyhow::Result<Quest> {
        let old_quest = self.find(id.clone()).await?;
        let quest = sqlx::query_as::<_, Quest>(
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

        Ok(quest)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        sqlx::query(
            r#"
                delete from quests where id=$1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// とりあえず一旦HashMapにデータを保存しておく
type QuestDatas = HashMap<String, Quest>;

#[derive(Debug, Clone)]
pub struct QuestRepositoryForMemory {
    store: Arc<RwLock<QuestDatas>>,
}

impl QuestRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<QuestDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<QuestDatas> {
        self.store.read().unwrap()
    }
}

#[async_trait]
impl QuestRepository for QuestRepositoryForMemory {
    async fn create(&self, payload: CreateQuest) -> anyhow::Result<Quest> {
        let mut store = self.write_store_ref();
        let id = nanoid!();
        let quest = Quest::new(
            id.clone(),
            payload.title,
            payload.description,
            payload.price,
            payload.difficulty,
            payload.num_participate,
            payload.num_clear,
        );
        store.insert(id, quest.clone());
        Ok(quest)
    }

    async fn find(&self, id: String) -> anyhow::Result<Quest> {
        let store = self.read_store_ref();
        let quest = store.get(&id).map(|quest| quest.clone()).unwrap();
        Ok(quest)
    }

    async fn all(&self) -> anyhow::Result<Vec<Quest>> {
        let store = self.read_store_ref();
        let quests = Vec::from_iter(store.values().cloned());
        Ok(quests)
    }

    async fn update(&self, id: String, payload: UpdateQuest) -> anyhow::Result<Quest> {
        let mut store = self.write_store_ref();
        let quest = store.get(&id).context(NotFound)?;
        let title = payload.title.unwrap_or(quest.title.clone());
        let description = payload.description.unwrap_or(quest.description.clone());
        let price = payload.price.unwrap_or(quest.price.clone());
        let difficulty = payload.difficulty.unwrap_or(quest.difficulty.clone());
        let num_participate = payload
            .num_participate
            .unwrap_or(quest.num_participate.clone());
        let num_clear = payload.num_clear.unwrap_or(quest.num_clear.clone());
        let quest = Quest::new(
            quest.id.clone(),
            title,
            description,
            price,
            difficulty,
            num_participate,
            num_clear,
        );
        store.insert(id, quest.clone());
        Ok(quest)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).context(NotFound)?;
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
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: i32, // 0ならFree
    #[sqlx(try_from = "String")]
    pub difficulty: Difficulty,
    pub num_participate: i32,
    pub num_clear: i32,
    // prize: String,
    // rating: u8,
    // created_at: DateTime<Local>,
    // milestones: Vec<Milestones>,
}

impl Quest {
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
        }
    }
}

// 各fieldが一致したとき==とみなす
impl PartialEq for Quest {
    fn eq(&self, other: &Quest) -> bool {
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
