use anyhow::Context;
use axum::async_trait;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    io::ErrorKind::NotFound,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::repositories::quest::{Difficulty, Quest};

#[async_trait]
pub trait UserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<UserEntity>;
    async fn login(&self, payload: LoginUser) -> anyhow::Result<UserEntity>;
    async fn find(&self, id: String) -> anyhow::Result<UserEntity>;
    async fn delete(&self, id: String) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct UserRepositoryForDb {
    pool: PgPool,
}

impl UserRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        UserRepositoryForDb { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryForDb {
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<UserEntity> {
        let row = sqlx::query_as::<_, UserFromRow>(
            r#"
                insert into users values ($1, $2, $3, $4)
                returning *
            "#,
        )
        .bind(nanoid!())
        .bind(payload.username)
        .bind(payload.email)
        .bind(payload.password)
        .fetch_one(&self.pool)
        .await?;

        let user = UserEntity::new(row.id, row.username, row.email, row.password);

        anyhow::Ok(user)
    }

    async fn login(&self, payload: LoginUser) -> anyhow::Result<UserEntity> {
        let user_row = sqlx::query_as::<_, UserFromRow>(
            r#"
                select * from users where username=$1 and password=$2;
            "#,
        )
        .bind(payload.username)
        .bind(payload.password)
        .fetch_one(&self.pool)
        .await?;

        let user_quest = sqlx::query_as::<_, UserWithQuestFromRow>(
            r#"
                select user_quests.*, quests.title as title, quests.description as description, quests.price as price, quests.difficulty as difficulty, quests.num_participate as num_participate, quests.num_clear as num_clear from user_quests
                    left outer join quests on user_quests.quest_id = quests.id
                    where user_id=$1;
            "#
        )
        .bind(user_row.id.clone())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| Vec::<UserWithQuestFromRow>::new())
        .unwrap();

        let quests = user_quest
            .iter()
            .map(|x| {
                Quest::new(
                    x.quest_id.clone(),
                    x.title.clone(),
                    x.description.clone(),
                    x.price.clone(),
                    x.difficulty.clone(),
                    x.num_participate.clone(),
                    x.num_clear.clone(),
                )
            })
            .collect::<Vec<Quest>>();

        let user = UserEntity {
            id: user_row.id.clone(),
            username: user_row.id.clone(),
            email: user_row.email.clone(),
            password: user_row.password.clone(),
            participate_quest: quests,
        };

        anyhow::Ok(user)
    }

    async fn find(&self, id: String) -> anyhow::Result<UserEntity> {
        let user_row = sqlx::query_as::<_, UserFromRow>(
            r#"
                select * from users where id=$1;
            "#,
        )
        .bind(id.clone())
        .fetch_one(&self.pool)
        .await?;

        let user_quest = sqlx::query_as::<_, UserWithQuestFromRow>(
            r#"
                select user_quests.*, quests.title as title, quests.description as description, quests.price as price, quests.difficulty as difficulty, quests.num_participate as num_participate, quests.num_clear as num_clear from user_quests
                    left outer join quests on user_quests.quest_id = quests.id
                    where user_id=$1;
            "#
        )
        .bind(id.clone())
        .fetch_all(&self.pool)
        .await
        .map_err(|_| Vec::<UserWithQuestFromRow>::new())
        .unwrap();

        let quests = user_quest
            .iter()
            .map(|x| {
                Quest::new(
                    x.quest_id.clone(),
                    x.title.clone(),
                    x.description.clone(),
                    x.price.clone(),
                    x.difficulty.clone(),
                    x.num_participate.clone(),
                    x.num_clear.clone(),
                )
            })
            .collect::<Vec<Quest>>();

        let user = UserEntity {
            id: user_row.id.clone(),
            username: user_row.id.clone(),
            email: user_row.email.clone(),
            password: user_row.password.clone(),
            participate_quest: quests,
        };

        anyhow::Ok(user)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        let tx = self.pool.begin().await?;

        // user_questsの削除
        sqlx::query(
            r#"
                delete from user_quests where use_id=$1
            "#,
        )
        .bind(id.clone())
        .execute(&self.pool)
        .await?;

        // userの削除
        sqlx::query(
            r#"
                delete from users where id=$1
            "#,
        )
        .bind(id.clone())
        .execute(&self.pool)
        .await?;

        tx.commit().await?;

        anyhow::Ok(())
    }
}

type UserDatas = HashMap<String, UserEntity>;

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
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<UserEntity> {
        let mut store = self.write_store_ref();
        let id = nanoid!();
        let user = UserEntity::new(
            id.clone(),
            payload.username,
            payload.email,
            payload.password,
        );
        store.insert(id, user.clone());
        anyhow::Ok(user)
    }

    async fn login(&self, payload: LoginUser) -> anyhow::Result<UserEntity> {
        let store = self.read_store_ref();
        let user_vec = store
            .values()
            .filter(|user| {
                (**user).username == payload.username && (**user).password == payload.password
            })
            .map(|user| user.clone())
            .collect::<Vec<UserEntity>>();
        let user = user_vec.get(0).unwrap();
        anyhow::Ok(user.clone())
    }

    async fn find(&self, id: String) -> anyhow::Result<UserEntity> {
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

#[derive(Debug, Clone, FromRow)]
struct UserFromRow {
    id: String,
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, FromRow)]
struct UserWithQuestFromRow {
    id: i32,
    user_id: String,
    quest_id: String,
    title: String,
    description: String,
    price: i32, // 0ならFree
    #[sqlx(try_from = "String")]
    difficulty: Difficulty,
    num_participate: i32,
    num_clear: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEntity {
    id: String,
    username: String,
    email: String,
    password: String,
    participate_quest: Vec<Quest>,
}

impl UserEntity {
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
