use anyhow::anyhow;
use axum::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

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

    #[cfg(test)]
    /// テスト用の簡易版コンストラクタ
    pub async fn with_url(url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(url).await?;
        Ok(UserRepositoryForDb::new(pool))
    }
}

#[async_trait]
impl UserRepository for UserRepositoryForDb {
    async fn register(&self, payload: RegisterUser) -> anyhow::Result<UserEntity> {
        let hashed_password = hash(payload.password, DEFAULT_COST)?;
        let row = sqlx::query_as::<_, UserFromRow>(
            r#"
                insert into users values ($1, $2, $3, $4)
                returning *
            "#,
        )
        .bind(nanoid!())
        .bind(payload.username)
        .bind(payload.email)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await?;

        let user = UserEntity::new(row.id, row.username, row.email);

        anyhow::Ok(user)
    }

    async fn login(&self, payload: LoginUser) -> anyhow::Result<UserEntity> {
        let user_row = sqlx::query_as::<_, UserFromRow>(
            r#"
                select * from users where email=$1;
            "#,
        )
        .bind(payload.email)
        .fetch_one(&self.pool)
        .await?;

        let verified = verify(payload.password, &user_row.password)?;
        if !verified {
            return Err(anyhow!("Invalid Password"));
        }

        let user = UserEntity {
            id: user_row.id.clone(),
            username: user_row.username.clone(),
            email: user_row.email.clone(),
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

        let user = UserEntity {
            id: user_row.id.clone(),
            username: user_row.username.clone(),
            email: user_row.email.clone(),
        };

        anyhow::Ok(user)
    }

    async fn delete(&self, id: String) -> anyhow::Result<()> {
        let tx = self.pool.begin().await?;

        // user_challengesの削除
        sqlx::query(
            r#"
                delete from user_completed_challenges where user_id=$1
            "#,
        )
        .bind(id.clone())
        .execute(&self.pool)
        .await?;

        // user_questsの削除
        sqlx::query(
            r#"
                delete from user_participating_quests where user_id=$1
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

#[derive(Debug, Clone, FromRow)]
struct UserFromRow {
    id: String,
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEntity {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl UserEntity {
    pub fn new(id: String, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
        }
    }
}

// usernameとemailが一致したときは==とみなす
// idと参加クエストが違っても同じユーザー
impl PartialEq for UserEntity {
    fn eq(&self, other: &UserEntity) -> bool {
        (self.username == other.username) && (self.email == other.email)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterUser {
    username: String,
    email: String,
    password: String,
}

#[cfg(test)]
impl RegisterUser {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            password,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    email: String,
    password: String,
}
