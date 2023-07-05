use aws_sdk_dynamodb::{types::AttributeValue, Client};
use std::collections::HashMap;
use tokio_stream::StreamExt as _;

pub struct DynamoDB {
    client: Client,
}

impl DynamoDB {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

/*
 * ============
 * "users" Table
 * ============
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserItem {
    pub id: String,
    pub email: String,
    pub name: String,
    pub hashed_password: String,
}

impl DynamoDB {
    pub const USER_TABLE_NAME: &'static str = "users";

    pub async fn put_user(&self, user: UserItem) -> anyhow::Result<()> {
        self.client
            .put_item()
            .table_name(Self::USER_TABLE_NAME)
            .item("UserId", AttributeValue::S(user.id))
            .item("UserEmail", AttributeValue::S(user.email))
            .item("UserName", AttributeValue::S(user.name))
            .item("UserPassword", AttributeValue::S(user.hashed_password))
            .send()
            .await?;
        Ok(())
    }

    fn map_item_to_user_item(item: &HashMap<String, AttributeValue>) -> UserItem {
        UserItem {
            id: item["UserId"].as_s().unwrap().clone(),
            email: item["UserEmail"].as_s().unwrap().clone(),
            name: item["UserName"].as_s().unwrap().clone(),
            hashed_password: item["UserPassword"].as_s().unwrap().clone(),
        }
    }

    pub async fn get_user_by_id(&self, id: String) -> anyhow::Result<Option<UserItem>> {
        let result = self
            .client
            .get_item()
            .table_name(Self::USER_TABLE_NAME)
            .key("UserId", AttributeValue::S(id))
            .send()
            .await?;
        let Some(item) = result.item() else {
            return Ok(None);
        };
        Ok(Some(Self::map_item_to_user_item(item)))
    }

    pub async fn get_user_by_email(&self, email: String) -> anyhow::Result<Option<UserItem>> {
        let result = self
            .client
            .get_item()
            .table_name(Self::USER_TABLE_NAME)
            .key("UserEmail", AttributeValue::S(email))
            .send()
            .await?;
        let Some(item) = result.item() else {
            return Ok(None);
        };
        Ok(Some(Self::map_item_to_user_item(item)))
    }

    pub async fn delete_user(&self, id: String) -> anyhow::Result<()> {
        self.client
            .delete_item()
            .table_name(Self::USER_TABLE_NAME)
            .key("UserId", AttributeValue::S(id))
            .send()
            .await?;
        Ok(())
    }
}

/*
 * ==============
 * "user_participating_quests" Table
 * ==============
 */
impl DynamoDB {
    pub const USER_PARTICIPATING_QUESTS_TABLE_NAME: &'static str = "user_participating_quests";

    pub async fn put_user_participate_quest(
        &self,
        user_id: String,
        quest_id: String,
    ) -> anyhow::Result<()> {
        self.client
            .put_item()
            .table_name(Self::USER_PARTICIPATING_QUESTS_TABLE_NAME)
            .item("UserId", AttributeValue::S(user_id))
            .item("QuestId", AttributeValue::S(quest_id))
            .send()
            .await?;
        Ok(())
    }

    pub async fn query_user_participate_quest_ids(
        &self,
        user_id: String,
    ) -> anyhow::Result<Vec<String>> {
        let result = self
            .client
            .query()
            .table_name(Self::USER_PARTICIPATING_QUESTS_TABLE_NAME)
            .key_condition_expression("UserId = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::S(user_id))
            .send()
            .await?;
        let Some(items) = result.items() else {
            return Ok(Vec::new());
        };
        let quest_ids = items
            .iter()
            .map(|item| item.get("QuestId").unwrap().as_s().unwrap().clone())
            .collect::<Vec<String>>();
        Ok(quest_ids)
    }
}

/*
 * ===========
 * "quests" Table
 * ===========
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl std::str::FromStr for Difficulty {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Self::Easy),
            "Normal" => Ok(Self::Normal),
            "Hard" => Ok(Self::Hard),
            _ => Err(anyhow::anyhow!("Invalid difficulty : {}", s)),
        }
    }
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        match self {
            Self::Easy => "Easy".to_string(),
            Self::Normal => "Normal".to_string(),
            Self::Hard => "Hard".to_string(),
        }
    }
}

impl DynamoDB {
    pub const QUEST_TABLE_NAME: &'static str = "quests";

    pub async fn put_quest(&self, quest: QuestItem) -> anyhow::Result<()> {
        self.client
            .put_item()
            .table_name(Self::QUEST_TABLE_NAME)
            .item("QuestId", AttributeValue::S(quest.id))
            .item("QuestTitle", AttributeValue::S(quest.title))
            .item("QuestDescription", AttributeValue::S(quest.description))
            .item("QuestPrice", AttributeValue::N(quest.price.to_string()))
            .item(
                "QuestDifficulty",
                AttributeValue::S(quest.difficulty.to_string()),
            )
            .send()
            .await?;
        Ok(())
    }

    fn map_item_to_quest_item(item: &HashMap<String, AttributeValue>) -> QuestItem {
        QuestItem {
            id: item["QuestId"].as_s().unwrap().clone(),
            title: item["QuestTitle"].as_s().unwrap().clone(),
            description: item["QuestDescription"].as_s().unwrap().clone(),
            price: item["QuestPrice"].as_n().unwrap().parse::<i32>().unwrap(),
            difficulty: item["QuestDifficulty"]
                .as_s()
                .unwrap()
                .parse::<Difficulty>()
                .unwrap(),
        }
    }

    pub async fn get_quest_by_id(&self, id: String) -> anyhow::Result<Option<QuestItem>> {
        let result = self
            .client
            .get_item()
            .table_name(Self::QUEST_TABLE_NAME)
            .key("QuestId", AttributeValue::S(id))
            .send()
            .await?;
        let Some(item) = result.item() else {
            return Ok(None);
        };
        Ok(Some(Self::map_item_to_quest_item(item)))
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<QuestItem>> {
        Ok(self
            .client
            .scan()
            .table_name(Self::QUEST_TABLE_NAME)
            .into_paginator()
            .items()
            .send()
            .map(|res| res.map(|i| Self::map_item_to_quest_item(&i)))
            .collect::<Result<Vec<QuestItem>, _>>()
            .await?)
    }

    pub async fn update_quest(&self, item: QuestItem) -> anyhow::Result<()> {
        self.client
            .update_item()
            .table_name(Self::QUEST_TABLE_NAME)
            .key("QuestId", AttributeValue::S(item.id))
            .update_expression(
                "SET QuestTitle = :title, QuestDescription = :description, QuestPrice = :price, QuestDifficulty = :difficulty",
            )
            .expression_attribute_values(
                ":title", AttributeValue::S(item.title))
            .expression_attribute_values(
                ":description", AttributeValue::S(item.description))
            .expression_attribute_values(
                ":price", AttributeValue::N(item.price.to_string()))
            .expression_attribute_values(
                ":difficulty", AttributeValue::S(item.difficulty.to_string()),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_quest(&self, id: String) -> anyhow::Result<()> {
        self.client
            .delete_item()
            .table_name(Self::QUEST_TABLE_NAME)
            .key("QuestId", AttributeValue::S(id))
            .send()
            .await?;
        Ok(())
    }
}

/*
 * ==============
 * "challenges" Table
 * ==============
 */
#[derive(Debug, Clone, PartialEq)]
pub struct ChallengeItem {
    id: String,
    quest_id: String,
    title: String,
    description: String,
    lat: f64,
    lon: f64,
}

impl DynamoDB {
    pub const CHALLENGE_TABLE_NAME: &'static str = "challenges";

    pub async fn put_challenge(&self, challenge: ChallengeItem) -> anyhow::Result<()> {
        self.client
            .put_item()
            .table_name(Self::CHALLENGE_TABLE_NAME)
            .item("ChallengeId", AttributeValue::S(challenge.id))
            .item("QuestId", AttributeValue::S(challenge.quest_id))
            .item("ChallengeTitle", AttributeValue::S(challenge.title))
            .item(
                "ChallengeDescription",
                AttributeValue::S(challenge.description),
            )
            .item("ChallengeLat", AttributeValue::N(challenge.lat.to_string()))
            .item("ChallengeLon", AttributeValue::N(challenge.lon.to_string()))
            .send()
            .await?;
        Ok(())
    }

    fn map_item_to_challenge_item(item: &HashMap<String, AttributeValue>) -> ChallengeItem {
        ChallengeItem {
            id: item["ChallengeId"].as_s().unwrap().clone(),
            quest_id: item["QuestId"].as_s().unwrap().clone(),
            title: item["ChallengeTitle"].as_s().unwrap().clone(),
            description: item["ChallengeDescription"].as_s().unwrap().clone(),
            lat: item["ChallengeLat"].as_n().unwrap().parse::<f64>().unwrap(),
            lon: item["ChallengeLon"].as_n().unwrap().parse::<f64>().unwrap(),
        }
    }

    pub async fn get_by_id(&self, id: String) -> anyhow::Result<Option<ChallengeItem>> {
        let result = self
            .client
            .get_item()
            .table_name(Self::CHALLENGE_TABLE_NAME)
            .key("ChallengeId", AttributeValue::S(id))
            .send()
            .await?;
        let Some(item) = result.item() else {
            return Ok(None);
        };
        Ok(Some(Self::map_item_to_challenge_item(item)))
    }

    pub async fn get_all_by_quest_id(
        &self,
        quest_id: String,
    ) -> anyhow::Result<Vec<ChallengeItem>> {
        let result = self
            .client
            .query()
            .table_name(Self::CHALLENGE_TABLE_NAME)
            .key_condition_expression("QuestId = :quest_id")
            .expression_attribute_values(":quest_id", AttributeValue::S(quest_id))
            .send()
            .await?;
        let Some(items) = result.items() else {
            return Ok(Vec::new());
        };
        let challenges = items
            .iter()
            .map(Self::map_item_to_challenge_item)
            .collect::<Vec<ChallengeItem>>();
        Ok(challenges)
    }
}

/// 実行前にdocker composeでdynamodb-localを起動しておく必要がある
#[cfg(all(test, feature = "db-tests"))]
mod tests {
    use super::*;

    async fn create_client() -> DynamoDB {
        let credentials = aws_sdk_dynamodb::config::Credentials::for_tests();
        let region = aws_sdk_dynamodb::config::Region::new("ap-northeast-1");
        let config = aws_sdk_dynamodb::Config::builder()
            .endpoint_url("http://localhost:4566")
            .credentials_provider(credentials)
            .region(region)
            .build();
        let client = Client::from_conf(config);
        DynamoDB { client }
    }

    #[tokio::test]
    async fn test_quest_crud() {
        let db = create_client().await;

        let quest = QuestItem {
            id: "test-quest".to_string(),
            title: "Test Quest".to_string(),
            description: "This is a test quest".to_string(),
            price: 100,
            difficulty: Difficulty::Easy,
        };
        db.put_quest(quest.clone()).await.unwrap();

        let queried_quest = db.get_quest_by_id(quest.id.clone()).await.unwrap();
        assert_eq!(queried_quest, Some(quest.clone()));

        let queried_quests = db.get_all().await.unwrap();
        assert_eq!(queried_quests.len(), 1);
        assert_eq!(queried_quests[0], quest);

        let updated_quest = QuestItem {
            difficulty: Difficulty::Normal,
            ..quest
        };
        db.update_quest(updated_quest.clone()).await.unwrap();

        let queried_quest = db.get_quest_by_id(updated_quest.id.clone()).await.unwrap();
        assert_eq!(queried_quest, Some(updated_quest.clone()));

        db.delete_quest(updated_quest.id.clone()).await.unwrap();
        let queried_quest = db.get_quest_by_id(updated_quest.id.clone()).await.unwrap();
        assert_eq!(queried_quest, None);
    }
}
