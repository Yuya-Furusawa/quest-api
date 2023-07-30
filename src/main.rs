mod handlers;
mod infras;
mod repositories;
mod services;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use http::{HeaderValue, Method};
use hyper::header::CONTENT_TYPE;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

use crate::handlers::{
    challenge::{create_challenge, find_challenge, find_challenge_by_quest_id},
    quest::{all_quests, create_quest, delete_quest, find_quest, update_quest},
    user::{auth_user, delete_user, find_user, login_user, register_user},
    user_challenge::complete_challenge,
    user_quest::participate_quest,
};
use crate::repositories::{
    challenge::{ChallengeRepository, ChallengeRepositoryForDb},
    quest::{QuestRepository, QuestRepositoryForDb},
    user::{UserRepository, UserRepositoryForDb},
    user_challenge::{UserChallengeRepository, UserChallengeRepositoryForDb},
    user_quest::{UserQuestRepository, UserQuestRepositoryForDb},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    let secret_key = env::var("JWT_SECRET_KEY").expect("undefined [JWT_SECRET_KEY]");

    let pool = PgPool::connect(database_url)
        .await
        .expect(&format!("fail connect database, url is [{}]", database_url));

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Failed to parse PORT");

    let app = create_app(
        QuestRepositoryForDb::new(pool.clone()),
        UserRepositoryForDb::new(pool.clone()),
        ChallengeRepositoryForDb::new(pool.clone()),
        UserQuestRepositoryForDb::new(pool.clone()),
        UserChallengeRepositoryForDb::new(pool.clone()),
        secret_key,
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<
    T: QuestRepository,
    S: UserRepository,
    U: ChallengeRepository,
    P: UserQuestRepository,
    Q: UserChallengeRepository,
>(
    quest_repository: T,
    user_repository: S,
    challenge_repository: U,
    userquest_repository: P,
    userchallenge_repository: Q,
    secret_key: String,
) -> Router {
    let user_routes = create_user_routes(user_repository, secret_key);
    let quest_routes = create_quest_routes(quest_repository, userquest_repository);
    let challenge_routes = create_challenge_routes(challenge_repository, userchallenge_repository);

    let origins = [
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "https://quest-web-cli.vercel.app"
            .parse::<HeaderValue>()
            .unwrap(),
    ];

    Router::new()
        .route("/", get(root))
        .nest("/", user_routes)
        .nest("/", quest_routes)
        .nest("/", challenge_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(vec![CONTENT_TYPE]),
        )
}

#[derive(Clone)]
pub struct UserHandlerState<T: UserRepository> {
    user_repository: Arc<T>,
    secret_key: String,
}

fn create_user_routes<T: UserRepository>(user_repository: T, secret_key: String) -> Router {
    let user_state = UserHandlerState {
        user_repository: Arc::new(user_repository),
        secret_key,
    };

    Router::new()
        .route("/register", post(register_user::<T>))
        .route("/login", post(login_user::<T>))
        .route("/users/:id", get(find_user::<T>).delete(delete_user::<T>))
        .route("/user/auth", get(auth_user::<T>))
        .layer(Extension(user_state))
}

fn create_quest_routes<T: QuestRepository, S: UserQuestRepository>(
    quest_repository: T,
    userquest_repository: S,
) -> Router {
    Router::new()
        .route("/quests", post(create_quest::<T>).get(all_quests::<T>))
        .route(
            "/quests/:id",
            get(find_quest::<T>)
                .patch(update_quest::<T>)
                .delete(delete_quest::<T>),
        )
        .route("/quests/:id/participate", post(participate_quest::<S>))
        .layer(Extension(Arc::new(quest_repository)))
        .layer(Extension(Arc::new(userquest_repository)))
}

fn create_challenge_routes<T: ChallengeRepository, S: UserChallengeRepository>(
    challenge_repository: T,
    userchallenge_repository: S,
) -> Router {
    Router::new()
        .route(
            "/challenges",
            post(create_challenge::<T>).get(find_challenge_by_quest_id::<T>),
        )
        .route("/challenges/:id", get(find_challenge::<T>))
        .route("/challenges/:id/complete", post(complete_challenge::<S>))
        .layer(Extension(Arc::new(challenge_repository)))
        .layer(Extension(Arc::new(userchallenge_repository)))
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
    use http::{header::SET_COOKIE, HeaderMap};
    use hyper::{self, StatusCode};
    use nanoid::nanoid;
    use tower::ServiceExt;

    use crate::repositories::{
        challenge::{Challenge, ChallengeRepositoryForMemory, CreateChallenge},
        quest::{CreateQuest, Difficulty, QuestEntity, QuestRepositoryForMemory},
        user::{RegisterUser, UserEntity, UserRepositoryForMemory},
        user_challenge::{CompleteChallenge, UserChallengeRepositoryForMemory},
        user_quest::{ParticipateQuest, UserQuestRepositoryForMemory},
    };

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

    async fn res_to_quest(res: Response) -> QuestEntity {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let quest: QuestEntity = serde_json::from_str(&body)
            .expect(&format!("cannot convert Quest instance. body: {}", body));
        quest
    }

    async fn res_to_user(res: Response) -> UserEntity {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        let user: UserEntity = serde_json::from_str(&body_str)
            .expect(&format!("cannot convert User instance. body: {}", body_str));
        user
    }

    async fn res_to_usercookie(res: Response) -> (UserEntity, HeaderMap) {
        let (parts, body) = res.into_parts();

        let bytes = hyper::body::to_bytes(body).await.unwrap();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        let user: UserEntity = serde_json::from_str(&body_str)
            .expect(&format!("cannot convert User instance. body: {}", body_str));

        let header_map = parts.headers;

        (user, header_map)
    }

    async fn res_to_challenge(res: Response) -> Challenge {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let challenge: Challenge = serde_json::from_str(&body).expect(&format!(
            "cannot convert ParticipateQuest instance. body: {}",
            body
        ));
        challenge
    }

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = Router::new()
            .route("/", get(root))
            .oneshot(req)
            .await
            .unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(body, "Hello World!")
    }

    #[tokio::test]
    async fn should_create_quest() {
        let expected = QuestEntity::new(
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
        let res = create_quest_routes(
            QuestRepositoryForMemory::new(),
            UserQuestRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let quest = res_to_quest(res).await;

        // idは異なる
        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_find_quest() {
        let quest_repository = QuestRepositoryForMemory::new();
        let expected = QuestEntity::new(
            nanoid!(),
            "Test Find Quest".to_string(),
            "This is a test of finding a quest.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );

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
        let res = create_quest_routes(quest_repository, UserQuestRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_all_quests() {
        let quest_repository = QuestRepositoryForMemory::new();
        let expected = QuestEntity::new(
            nanoid!(),
            "Test All Quests".to_string(),
            "This is a test of finding all quests.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );
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
        let res = create_quest_routes(quest_repository, UserQuestRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quests: Vec<QuestEntity> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Quest instance. body {}", body));
        assert_eq!(vec![expected], quests);
    }

    #[tokio::test]
    async fn should_update_quest() {
        let quest_repository = QuestRepositoryForMemory::new();
        let expected = QuestEntity::new(
            nanoid!(),
            "Test Update Quests".to_string(),
            "This is a test of updating a quest.".to_string(),
            0,
            Difficulty::Normal,
            12345,
            123,
        );
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
        let res = create_quest_routes(quest_repository, UserQuestRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_delete_quest() {
        let quest_repository = QuestRepositoryForMemory::new();
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
        let res = create_quest_routes(quest_repository, UserQuestRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_register_user() {
        let user_repository = UserRepositoryForMemory::new();
        let expected = UserEntity::new(
            nanoid!(),
            "Test User".to_string(),
            "test@test.com".to_string(),
        );

        let req = build_req_with_json(
            "/register",
            Method::POST,
            r#"{
                "username": "Test User",
                "email": "test@test.com",
                "password": "password"
            }"#
            .to_string(),
        );

        let secret_key = "secret_key".to_string();

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .expect("failed to register user");

        let (user, header_map) = res_to_usercookie(res).await;

        assert_eq!(expected, user);
        assert!(header_map.contains_key(SET_COOKIE));
    }

    #[tokio::test]
    async fn should_login_user() {
        let user_repository = UserRepositoryForMemory::new();
        let created_user = user_repository
            .register(RegisterUser::new(
                "Test User".to_string(),
                "test@test.com".to_string(),
                "password".to_string(),
            ))
            .await
            .expect("failed to create user");

        let req = build_req_with_json(
            "/login",
            Method::POST,
            r#"{
                "email": "test@test.com",
                "password": "password"
            }"#
            .to_string(),
        );

        let secret_key = "secret_key".to_string();

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .expect("failed to login user");
        let (user, header_map) = res_to_usercookie(res).await;

        assert_eq!(created_user, user);
        assert!(header_map.contains_key(SET_COOKIE));
    }

    #[tokio::test]
    async fn should_find_user() {
        let user_repository = UserRepositoryForMemory::new();
        let created_user = user_repository
            .register(RegisterUser::new(
                "Test User".to_string(),
                "test@test.com".to_string(),
                "password".to_string(),
            ))
            .await
            .expect("failed to create user");

        let req_path = format!("{}{}", "/users/", created_user.id);
        let req = build_req_with_empty(&req_path, Method::GET);

        let secret_key = "secret_key".to_string();

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .expect("failed to find user");
        let user = res_to_user(res).await;

        assert_eq!(created_user, user);
    }

    #[tokio::test]
    async fn should_delete_user() {
        let user_repository = UserRepositoryForMemory::new();
        let creared_user = user_repository
            .register(RegisterUser::new(
                "Test User".to_string(),
                "test@test.com".to_string(),
                "password".to_string(),
            ))
            .await
            .expect("failed to create user");

        let req_path = format!("{}{}", "/users/", creared_user.id);
        let req = build_req_with_empty(&req_path, Method::DELETE);

        let secret_key = "secret_key".to_string();

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_participate_quest() {
        let repository = UserQuestRepositoryForMemory::new();

        let expected = ParticipateQuest {
            user_id: "test".to_string(),
            quest_id: "test".to_string(),
        };

        let req_path = format!("/quests/{}/participate", "test");

        let req = build_req_with_json(
            &req_path,
            Method::POST,
            r#"{
                "user_id": "test"
            }"#
            .to_string(),
        );

        create_quest_routes(QuestRepositoryForMemory::new(), repository.clone())
            .oneshot(req)
            .await
            .unwrap();

        let result = repository.read_stored_value();

        assert_eq!(expected, result[0])
    }

    #[tokio::test]
    async fn should_create_challenge() {
        let expected = Challenge::new(
            nanoid!(),
            "Test Challenge".to_string(),
            "This is a test challenge".to_string(),
            "test_id".to_string(),
            35.6895,
            139.6917,
        );

        let req = build_req_with_json(
            "/challenges",
            Method::POST,
            r#"{
                "name": "Test Challenge",
                "description": "This is a test challenge",
                "quest_id": "test_id",
                "latitude": 35.6895,
                "longitude": 139.6917
            }"#
            .to_string(),
        );

        let res = create_challenge_routes(
            ChallengeRepositoryForMemory::new(),
            UserChallengeRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();

        let result = res_to_challenge(res).await;

        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn should_find_challenge() {
        let challenge_repository = ChallengeRepositoryForMemory::new();
        let created_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                "test_id".to_string(),
                35.6895,
                139.6917,
            ))
            .await
            .expect("failed to create challenge");

        let req_path = format!("{}{}", "/challenges/", created_challenge.id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_challenge_routes(
            challenge_repository,
            UserChallengeRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .expect("failed to find challenge");
        let challenge = res_to_challenge(res).await;

        assert_eq!(created_challenge, challenge)
    }

    #[tokio::test]
    async fn should_find_challnege_by_quest_id() {
        let challenge_repository = ChallengeRepositoryForMemory::new();
        let created_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                "test_id".to_string(),
                35.6895,
                139.6917,
            ))
            .await
            .expect("failed to create challenge");

        let req_path = format!("{}?quest_id={}", "/challenges", created_challenge.quest_id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_challenge_routes(
            challenge_repository,
            UserChallengeRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .expect("failed to find challenge");

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let challenges: Vec<Challenge> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Challenge instance. body {}", body));

        assert_eq!(vec![created_challenge], challenges)
    }

    #[tokio::test]
    async fn should_complete_challenge() {
        let repository = UserChallengeRepositoryForMemory::new();

        let expected = CompleteChallenge {
            user_id: "test".to_string(),
            challenge_id: "test".to_string(),
        };

        let path = format!("/challenges/{}/complete", "test");

        let req = build_req_with_json(
            &path,
            Method::POST,
            r#"{
                "user_id": "test"
            }"#
            .to_string(),
        );

        create_challenge_routes(ChallengeRepositoryForMemory::new(), repository.clone())
            .oneshot(req)
            .await
            .unwrap();

        let result = repository.read_stored_value();

        assert_eq!(expected, result[0])
    }
}
