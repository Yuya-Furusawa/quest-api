use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[async_trait]
pub trait UserQuestRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn save_quest_participate_event(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()>;
    async fn get_participated_quests_by_user_id(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct UserQuestRepositoryForDb {
    pool: PgPool,
}

impl UserQuestRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        UserQuestRepositoryForDb { pool }
    }

    #[cfg(test)]
    /// テスト用の簡易版コンストラクタ
    pub async fn with_url(url: &str) -> Self {
        let pool = PgPool::connect(url).await.unwrap();
        UserQuestRepositoryForDb::new(pool)
    }

    #[cfg(test)]
    pub async fn query_user_participating_quests(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>> {
        let quests = sqlx::query_as::<_, ParticipateQuest>(
            r#"
                select * from user_participating_quests where user_id = $1;
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(quests.into_iter().map(|c| c.quest_id).collect())
    }
}

#[async_trait]
impl UserQuestRepository for UserQuestRepositoryForDb {
    async fn save_quest_participate_event(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()> {
        sqlx::query_as::<_, ParticipateQuest>(
            r#"
				insert into user_participating_quests (user_id, quest_id) values ($1, $2)
				returning *
			"#,
        )
        .bind(user_id)
        .bind(quest_id)
        .fetch_one(&self.pool)
        .await?;

        anyhow::Ok(())
    }

    async fn get_participated_quests_by_user_id(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>> {
        let quests = sqlx::query_as::<_, UserQuestFromRow>(
            r#"
                select * from user_participating_quests where user_id=$1;
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| Vec::<String>::new())
        .unwrap();

        let quest_ids = quests.iter().map(|x| x.quest_id.clone()).collect();

        anyhow::Ok(quest_ids)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
struct UserQuestFromRow {
    id: i32,
    user_id: String,
    quest_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
pub struct ParticipateQuest {
    pub user_id: String,
    pub quest_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticipateQuestPayload {
    pub user_id: String,
}
