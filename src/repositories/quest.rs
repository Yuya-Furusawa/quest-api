use axum::async_trait;
use anyhow::{Ok, Context};
use nanoid::nanoid;
use serde::{Serialize, Deserialize};
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

// とりあえず一旦HashMapにデータを保存しておく
type QuestDatas = HashMap<String, Quest>;

#[derive(Debug, Clone)]
pub struct QuestRepositoryForMemory {
    store: Arc<RwLock<QuestDatas>>
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
            payload.num_clear
        );
        store.insert(id, quest.clone());
        Ok(quest)
    }

    async fn find(&self, id: String) -> anyhow::Result<Quest> {
        let store = self.read_store_ref();
        let quest = store
        .get(&id)
        .map(|quest| quest.clone())
        .unwrap();
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
        let num_participate = payload.num_participate.unwrap_or(quest.num_participate.clone());
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

    async fn delete(&self, id:String) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).context(NotFound)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: u32, // 0ならFree
    pub difficulty: Difficulty,
    pub num_participate: u32,
    pub num_clear: u32,
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
        price: u32,
        difficulty: Difficulty,
        num_participate: u32,
        num_clear: u32,
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
    price: u32, // 0ならFree
    difficulty: Difficulty,
    num_participate: u32,
    num_clear: u32,
}

impl CreateQuest {
    pub fn new(
        title: String,
        description: String,
        price: u32,
        difficulty: Difficulty,
        num_participate: u32,
        num_clear: u32,
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
    price: Option<u32>,
    difficulty: Option<Difficulty>,
    num_participate: Option<u32>,
    num_clear: Option<u32>,
}
