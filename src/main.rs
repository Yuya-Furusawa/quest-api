mod handlers;
mod repositories;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};

use crate::handlers::{
    quest::{all_quests, create_quest, delete_quest, find_quest, update_quest},
    user::{delete_user, find_user, login_user, register_user},
};
use crate::repositories::{
    quest::{QuestRepository, QuestRepositoryForDb},
    user::{UserRepository, UserRepositoryForMemory},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    let pool = PgPool::connect(database_url)
        .await
        .expect(&format!("fail connect database, url is [{}]", database_url));

    let app = create_app(
        QuestRepositoryForDb::new(pool.clone()),
        UserRepositoryForMemory::new(),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: QuestRepository, S: UserRepository>(
    quest_repository: T,
    user_repository: S,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/quests", post(create_quest::<T>).get(all_quests::<T>))
        .route(
            "/quests/:id",
            get(find_quest::<T>)
                .patch(update_quest::<T>)
                .delete(delete_quest::<T>),
        )
        .route("/register", post(register_user::<S>))
        .route("/login", get(login_user::<S>))
        .route("/users/:id", get(find_user::<S>).delete(delete_user::<S>))
        .layer(Extension(Arc::new(quest_repository)))
        .layer(Extension(Arc::new(user_repository)))
}

async fn root() -> &'static str {
    "Hello World!"
}

#[cfg(test)]
mod test {
    use super::*;

    use axum::{
        body::Body,
        http::{header, Method, Request},
        response::Response,
    };
    use hyper::{self, StatusCode};
    use nanoid::nanoid;
    use tower::ServiceExt;

    use crate::repositories::quest::{CreateQuest, Difficulty, Quest, QuestRepositoryForMemory};

    fn build_req_with_empty(path: &str, method: Method) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    fn build_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    async fn res_to_quest(res: Response) -> Quest {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let quest: Quest = serde_json::from_str(&body)
            .expect(&format!("cannot convert Quest instance. body: {}", body));
        quest
    }

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(
            QuestRepositoryForMemory::new(),
            UserRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(body, "Hello World!")
    }

    #[tokio::test]
    async fn should_create_quest() {
        let expected = Quest::new(
            nanoid!(),
            "Test Create Quest".to_string(),
            "This is a test of creating a quest.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );

        let req = build_req_with_json(
            "/quests",
            Method::POST,
            r#"{
                "title": "Test Create Quest",
                "description": "This is a test of creating a quest.",
                "price": 0,
                "difficulty": "Normal",
                "num_participate": 12345,
                "num_clear": 123
             }"#
            .to_string(),
        );
        let res = create_app(
            QuestRepositoryForMemory::new(),
            UserRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let quest = res_to_quest(res).await;

        // id????????????
        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_find_quest() {
        let expected = Quest::new(
            nanoid!(),
            "Test Find Quest".to_string(),
            "This is a test of finding a quest.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );

        let quest_repository = QuestRepositoryForMemory::new();
        let user_repository = UserRepositoryForMemory::new();
        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Find Quest".to_string(),
                "This is a test of finding a quest.".to_string(),
                0,
                Difficulty::Normal,
                12345,
                123,
            ))
            .await
            .expect("failed to create quest");

        let req_path = format!("{}{}", "/quests/", created_quest.id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_app(quest_repository, user_repository)
            .oneshot(req)
            .await
            .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_all_quests() {
        let expected = Quest::new(
            nanoid!(),
            "Test All Quests".to_string(),
            "This is a test of finding all quests.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );

        let quest_repository = QuestRepositoryForMemory::new();
        let user_repository = UserRepositoryForMemory::new();
        quest_repository
            .create(CreateQuest::new(
                "Test All Quests".to_string(),
                "This is a test of finding all quests.".to_string(),
                0,
                Difficulty::Normal,
                12345,
                123,
            ))
            .await
            .expect("failed to create quest");

        let req = build_req_with_empty("/quests", Method::GET);
        let res = create_app(quest_repository, user_repository)
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quests: Vec<Quest> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Quest instance. body {}", body));
        assert_eq!(vec![expected], quests);
    }

    #[tokio::test]
    async fn should_update_quest() {
        let expected = Quest::new(
            nanoid!(),
            "Test Update Quests".to_string(),
            "This is a test of updating a quest.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );

        let quest_repository = QuestRepositoryForMemory::new();
        let user_repository = UserRepositoryForMemory::new();
        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Update Quests Before".to_string(),
                "This is a dummy quest before updating.".to_string(),
                0,
                Difficulty::Normal,
                12345,
                123,
            ))
            .await
            .expect("failed to create quest");

        let req_path = format!("{}{}", "/quests/", created_quest.id);
        let req = build_req_with_json(
            &req_path,
            Method::PATCH,
            r#"{
                "title": "Test Update Quests",
                "description": "This is a test of updating a quest."
             }"#
            .to_string(),
        );
        let res = create_app(quest_repository, user_repository)
            .oneshot(req)
            .await
            .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_delete_quest() {
        let quest_repository = QuestRepositoryForMemory::new();
        let user_repository = UserRepositoryForMemory::new();
        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Delete Quests".to_string(),
                "This is a test of deleting a quest.".to_string(),
                0,
                Difficulty::Normal,
                12345,
                123,
            ))
            .await
            .expect("failed to create quest");

        let req_path = format!("{}{}", "/quests/", created_quest.id);
        let req = build_req_with_empty(&req_path, Method::DELETE);
        let res = create_app(quest_repository, user_repository)
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
