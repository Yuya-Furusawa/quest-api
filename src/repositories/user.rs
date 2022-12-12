use anyhow::Context;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::ErrorKind::NotFound,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::repositories::quest::Quest;

#[async_trait]
pub trait UserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<User>;
    async fn login(&self, payload: LoginUser) -> anyhow::Result<User>;
    async fn find(&self, id: String) -> anyhow::Result<User>;
    async fn delete(&self, id: String) -> anyhow::Result<()>;
}

type UserDatas = HashMap<String, User>;

#[derive(Debug, Clone)]
pub struct UserRepositoryForMemory {
    store: Arc<RwLock<UserDatas>>,
}

impl UserRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<UserDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<UserDatas> {
        self.store.read().unwrap()
    }
}

#[async_trait]
impl UserRepository for UserRepositoryForMemory {
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<User> {
        let mut store = self.write_store_ref();
        let id = nanoid!();
        let user = User::new(
            id.clone(),
            payload.username,
            payload.email,
            payload.password,
        );
        store.insert(id, user.clone());
        anyhow::Ok(user)
    }

    async fn login(&self, payload: LoginUser) -> anyhow::Result<User> {
        let store = self.read_store_ref();
        let user_vec = store
            .values()
            .filter(|user| {
                (**user).username == payload.username && (**user).password == payload.password
            })
            .map(|user| user.clone())
            .collect::<Vec<User>>();
        let user = user_vec.get(0).unwrap();
        anyhow::Ok(user.clone())
    }

    async fn find(&self, id: String) -> anyhow::Result<User> {
        let store = self.read_store_ref();
        let user = store.get(&id).map(|user| user.clone()).unwrap();
        anyhow::Ok(user)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).context(NotFound)?;
        anyhow::Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    id: String,
    username: String,
    email: String,
    password: String,
    participate_quest: Vec<Quest>,
}

impl User {
    fn new(id: String, username: String, email: String, password: String) -> Self {
        Self {
            id,
            username,
            email,
            password,
            participate_quest: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    username: String,
    password: String,
}
