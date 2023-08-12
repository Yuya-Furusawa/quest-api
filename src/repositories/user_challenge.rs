use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[async_trait]
pub trait UserChallengeRepository: Clone + Send + Sync + 'static {
    async fn save_challenge_complete_event(
        &self,
        user_id: String,
        challenge_id: String,
    ) -> anyhow::Result<()>;
    async fn get_completed_challenges_by_user_id(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct UserChallengeRepositoryForDb {
    pool: PgPool,
}

impl UserChallengeRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        UserChallengeRepositoryForDb { pool }
    }

    #[cfg(test)]
    /// テスト用の簡易版コンストラクタ
    pub async fn with_url(url: &str) -> Self {
        let pool = PgPool::connect(url).await.unwrap();
        UserChallengeRepositoryForDb::new(pool)
    }

    #[cfg(test)]
    /// テスト用の確認メソッド
    pub async fn query_user_completed_challenges(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>> {
        let challenges = sqlx::query_as::<_, CompleteChallenge>(
            r#"
                select * from user_completed_challenges where user_id = $1;
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(challenges.into_iter().map(|c| c.challenge_id).collect())
    }
}

#[async_trait]
impl UserChallengeRepository for UserChallengeRepositoryForDb {
    async fn save_challenge_complete_event(
        &self,
        user_id: String,
        challenge_id: String,
    ) -> anyhow::Result<()> {
        sqlx::query_as::<_, CompleteChallenge>(
            r#"
                insert into user_completed_challenges (user_id, challenge_id) values ($1, $2)
                returning *
            "#,
        )
        .bind(user_id)
        .bind(challenge_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(())
    }

    async fn get_completed_challenges_by_user_id(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>> {
        let challenges = sqlx::query_as::<_, UserChallengeFromRow>(
            r#"
                select * from user_completed_challenges where user_id=$1;
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| Vec::<String>::new())
        .unwrap();

        let quest_ids = challenges.iter().map(|x| x.challenge_id.clone()).collect();

        anyhow::Ok(quest_ids)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
struct UserChallengeFromRow {
    id: i32,
    user_id: String,
    challenge_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
pub struct CompleteChallenge {
    pub user_id: String,
    pub challenge_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompleteChallengePayload {
    pub user_id: String,
}
