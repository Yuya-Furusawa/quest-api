mod handlers;
mod infras;
mod middleware;
mod repositories;
mod services;

use axum::{
    extract::Extension,
    middleware::from_fn,
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
    user_challenge::{complete_challenge, get_completed_challenges},
    user_quest::{get_participated_quests, participate_quest},
};
use crate::middleware::auth::auth_middleware;
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
    let user_routes = create_user_routes(user_repository, secret_key.clone());
    let quest_routes = create_quest_routes(
        quest_repository,
        userquest_repository.clone(),
        secret_key.clone(),
    );
    let challenge_routes = create_challenge_routes(
        challenge_repository,
        userchallenge_repository.clone(),
        secret_key.clone(),
    );
    let user_info_routes = create_user_info_routes(
        userquest_repository.clone(),
        userchallenge_repository.clone(),
        secret_key,
    );

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
        .nest("/", user_info_routes)
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
        secret_key: secret_key.clone(),
    };

    let auth_routes = Router::new()
        .route("/users/:id", get(find_user::<T>).delete(delete_user::<T>))
        .route("/user/auth", get(auth_user::<T>))
        .layer(Extension(user_state.clone()))
        .layer(from_fn(move |req, next| {
            auth_middleware(secret_key.clone(), req, next)
        }));

    let non_auth_routes = Router::new()
        .route("/register", post(register_user::<T>))
        .route("/login", post(login_user::<T>))
        .layer(Extension(user_state));

    Router::new().merge(auth_routes).merge(non_auth_routes)
}

fn create_quest_routes<T: QuestRepository, S: UserQuestRepository>(
    quest_repository: T,
    userquest_repository: S,
    secret_key: String,
) -> Router {
    let auth_routes = Router::new()
        .route("/quests/:id/participate", post(participate_quest::<S>))
        .layer(from_fn(move |req, next| {
            auth_middleware(secret_key.clone(), req, next)
        }));

    let non_auth_routes = Router::new()
        .route("/quests", post(create_quest::<T>).get(all_quests::<T>))
        .route(
            "/quests/:id",
            get(find_quest::<T>)
                .patch(update_quest::<T>)
                .delete(delete_quest::<T>),
        );

    Router::new()
        .merge(auth_routes)
        .merge(non_auth_routes)
        .layer(Extension(Arc::new(quest_repository)))
        .layer(Extension(Arc::new(userquest_repository)))
}

fn create_challenge_routes<T: ChallengeRepository, S: UserChallengeRepository>(
    challenge_repository: T,
    userchallenge_repository: S,
    secret_key: String,
) -> Router {
    let auth_routes = Router::new()
        .route("/challenges/:id/complete", post(complete_challenge::<S>))
        .layer(from_fn(move |req, next| {
            auth_middleware(secret_key.clone(), req, next)
        }));

    let non_auth_routes = Router::new()
        .route(
            "/challenges",
            post(create_challenge::<T>).get(find_challenge_by_quest_id::<T>),
        )
        .route("/challenges/:id", get(find_challenge::<T>));

    Router::new()
        .merge(auth_routes)
        .merge(non_auth_routes)
        .layer(Extension(Arc::new(challenge_repository)))
        .layer(Extension(Arc::new(userchallenge_repository)))
}

#[derive(Clone)]
pub struct UserInfoHandlerState<T: UserQuestRepository, S: UserChallengeRepository> {
    userquest_repository: Arc<T>,
    userchallenge_repository: Arc<S>,
}

fn create_user_info_routes<T: UserQuestRepository, S: UserChallengeRepository>(
    userquest_repository: T,
    userchallenge_repository: S,
    secret_key: String,
) -> Router {
    let user_info_state = UserInfoHandlerState {
        userquest_repository: Arc::new(userquest_repository),
        userchallenge_repository: Arc::new(userchallenge_repository),
    };

    Router::new()
        .route(
            "/me/participated_quests",
            get(get_participated_quests::<T, S>),
        )
        .route(
            "/me/completed_challenges",
            get(get_completed_challenges::<T, S>),
        )
        .layer(Extension(user_info_state))
        .layer(from_fn(move |req, next| {
            auth_middleware(secret_key.clone(), req, next)
        }))
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
    use chrono::{Duration, Utc};
    use http::{header::SET_COOKIE, HeaderMap};
    use hyper::{self, StatusCode};
    use nanoid::nanoid;
    use tower::ServiceExt;

    use crate::repositories::{
        challenge::{Challenge, CreateChallenge},
        quest::{CreateQuest, QuestEntity},
        user::{RegisterUser, UserEntity},
    };
    use crate::services::user::create_jwt;

    const DB_URL_FOR_TEST: &str = "postgres://admin:admin@localhost:5432/quests";

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

    fn build_req_with_cookie(path: &str, method: Method, cookie: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header("Cookie", cookie)
            .body(Body::empty())
            .unwrap()
    }

    fn build_req_with_json_cookie(
        path: &str,
        method: Method,
        json_body: String,
        cookie: &str,
    ) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header("Cookie", cookie)
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
        );

        let req = build_req_with_json(
            "/quests",
            Method::POST,
            r#"{
                "title": "Test Create Quest",
                "description": "This is a test of creating a quest."
             }"#
            .to_string(),
        );
        let res = create_quest_routes(
            QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
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
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let expected = QuestEntity::new(
            nanoid!(),
            "Test Find Quest".to_string(),
            "This is a test of finding a quest.".to_string(),
        );

        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Find Quest".to_string(),
                "This is a test of finding a quest.".to_string(),
            ))
            .await
            .expect("failed to create quest");

        let req_path = format!("{}{}", "/quests/", created_quest.id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_quest_routes(
            quest_repository,
            UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_all_quests() {
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let expected = QuestEntity::new(
            nanoid!(),
            "Test All Quests".to_string(),
            "This is a test of finding all quests.".to_string(),
        );
        quest_repository
            .create(CreateQuest::new(
                "Test All Quests".to_string(),
                "This is a test of finding all quests.".to_string(),
            ))
            .await
            .expect("failed to create quest");

        let req = build_req_with_empty("/quests", Method::GET);
        let res = create_quest_routes(
            quest_repository.clone(),
            UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quests: Vec<QuestEntity> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Quest instance. body {}", body));
        assert_eq!(vec![expected.clone()], quests);
    }

    #[tokio::test]
    async fn should_update_quest() {
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let expected = QuestEntity::new(
            nanoid!(),
            "Test Update Quests".to_string(),
            "This is a test of updating a quest.".to_string(),
        );
        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Update Quests Before".to_string(),
                "This is a dummy quest before updating.".to_string(),
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
        let res = create_quest_routes(
            quest_repository,
            UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let quest = res_to_quest(res).await;

        assert_eq!(expected, quest);
    }

    #[tokio::test]
    async fn should_delete_quest() {
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let created_quest = quest_repository
            .create(CreateQuest::new(
                "Test Delete Quests".to_string(),
                "This is a test of deleting a quest.".to_string(),
            ))
            .await
            .expect("failed to create quest");

        let req_path = format!("{}{}", "/quests/", created_quest.id);
        let req = build_req_with_empty(&req_path, Method::DELETE);
        let res = create_quest_routes(
            quest_repository,
            UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();

        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_register_user() {
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
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
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
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
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let created_user = user_repository
            .register(RegisterUser::new(
                "Test User".to_string(),
                "test@test.com".to_string(),
                "password".to_string(),
            ))
            .await
            .expect("failed to create user");

        let secret_key = "secret_key".to_string();
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let token = create_jwt(&created_user.id, iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        let req_path = format!("{}{}", "/users/", created_user.id);
        let req = build_req_with_cookie(&req_path, Method::GET, &cookie_header);

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .expect("failed to find user");
        let user = res_to_user(res).await;

        assert_eq!(created_user, user);
    }

    #[tokio::test]
    async fn should_delete_user() {
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let created_user = user_repository
            .register(RegisterUser::new(
                "Test User".to_string(),
                "test@test.com".to_string(),
                "password".to_string(),
            ))
            .await
            .expect("failed to create user");

        let secret_key = "secret_key".to_string();
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let token = create_jwt(&created_user.id, iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        let req_path = format!("{}{}", "/users/", created_user.id);
        let req = build_req_with_cookie(&req_path, Method::DELETE, &cookie_header);

        let res = create_user_routes(user_repository, secret_key)
            .oneshot(req)
            .await
            .unwrap();

        let status = res.status();

        assert_eq!(StatusCode::NO_CONTENT, status);
    }

    #[tokio::test]
    async fn should_participate_quest() {
        // 事前準備
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_user = user_repository
            .unwrap()
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_quest = quest_repository
            .create(CreateQuest::new(
                "Test Quest".to_string(),
                "This is a test quest.".to_string(),
            ))
            .await
            .unwrap();

        // テスト対象
        let repository = UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;

        let secret_key = "secret_key".to_string();
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let token = create_jwt(&test_user.id, iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        let req_path = format!("/quests/{}/participate", test_quest.id);

        let req = build_req_with_json_cookie(
            &req_path,
            Method::POST,
            format!("{{\"user_id\": \"{}\" }}", test_user.id).to_string(),
            &cookie_header,
        );

        create_quest_routes(
            QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            repository.clone(),
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();

        let result = repository
            .query_user_participating_quests(test_user.id)
            .await
            .unwrap();

        assert_eq!(vec![test_quest.id], result);
    }

    #[tokio::test]
    async fn should_get_participated_quests() {
        // ユーザーの作成
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let test_user = user_repository
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();

        // クエストの作成
        let quest_repository = QuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_quest = quest_repository
            .create(CreateQuest::new(
                "Test Quest".to_string(),
                "This is a test quest.".to_string(),
            ))
            .await
            .unwrap();

        // クエスト参加を保存する
        let userquest_repository = UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let _ = userquest_repository
            .save_quest_participate_event(test_user.id.clone(), test_quest.id.clone())
            .await;

        // 認証のためにトークン作成
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let secret_key = "secret-key".to_string();
        let token = create_jwt(&test_user.id.clone(), iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        // テスト対象
        let userchallenge_repository =
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let req = build_req_with_cookie("/me/participated_quests", Method::GET, &cookie_header);
        let res =
            create_user_info_routes(userquest_repository, userchallenge_repository, secret_key)
                .oneshot(req)
                .await
                .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quest_ids: Vec<String> = serde_json::from_str(&body).expect(&format!(
            "cannot convert Vec<String> instance. body {}",
            body
        ));
        assert_eq!(vec![test_quest.id.clone()], quest_ids);
    }

    #[tokio::test]
    async fn should_return_empty_vec_when_zero_patricipated_quest() {
        // ユーザーの作成
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let test_user = user_repository
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();

        // 認証のためにトークン作成
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let secret_key = "secret-key".to_string();
        let token = create_jwt(&test_user.id.clone(), iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        // テスト対象
        let userquest_repository = UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let userchallenge_repository =
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let req = build_req_with_cookie("/me/participated_quests", Method::GET, &cookie_header);
        let res =
            create_user_info_routes(userquest_repository, userchallenge_repository, secret_key)
                .oneshot(req)
                .await
                .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quest_ids: Vec<String> = serde_json::from_str(&body).expect(&format!(
            "cannot convert Vec<String> instance. body {}",
            body
        ));
        assert_eq!(Vec::<String>::new(), quest_ids);
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
            "Test Stamp".to_string(),
            "test-stamp-image-color".to_string(),
            "test-stamp-image-gray".to_string(),
            "This is a test stamp".to_string(),
        );

        let req = build_req_with_json(
            "/challenges",
            Method::POST,
            r#"{
                "name": "Test Challenge",
                "description": "This is a test challenge",
                "quest_id": "test_id",
                "latitude": 35.6895,
                "longitude": 139.6917,
                "stamp_name": "Test Stamp",
                "stamp_color_image_url": "test-stamp-image-color",
                "stamp_gray_image_url": "test-stamp-image-gray",
                "flavor_text": "This is a test stamp"
            }"#
            .to_string(),
        );

        let res = create_challenge_routes(
            ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();

        let result = res_to_challenge(res).await;

        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn should_find_challenge() {
        let challenge_repository = ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let created_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                "test_id".to_string(),
                35.6895,
                139.6917,
                "Test Stamp".to_string(),
                "test-stamp-image-color".to_string(),
                "test-stamp-image-gray".to_string(),
                "This is a test stamp".to_string(),
            ))
            .await
            .expect("failed to create challenge");

        let req_path = format!("{}{}", "/challenges/", created_challenge.id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_challenge_routes(
            challenge_repository,
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .expect("failed to find challenge");
        let challenge = res_to_challenge(res).await;

        assert_eq!(created_challenge, challenge)
    }

    #[tokio::test]
    async fn should_find_challnege_by_quest_id() {
        let challenge_repository = ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let created_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                nanoid::nanoid!(),
                35.6895,
                139.6917,
                "Test Stamp".to_string(),
                "test-stamp-image-color".to_string(),
                "test-stamp-image-gray".to_string(),
                "This is a test stamp".to_string(),
            ))
            .await
            .expect("failed to create challenge");

        let req_path = format!("{}?quest_id={}", "/challenges", created_challenge.quest_id);
        let req = build_req_with_empty(&req_path, Method::GET);
        let res = create_challenge_routes(
            challenge_repository,
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            "secret_key".to_string(),
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
        // 事前準備
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_user = user_repository
            .unwrap()
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();
        let challenge_repository = ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                "test_id".to_string(),
                35.6895,
                139.6917,
                "Test Stamp".to_string(),
                "test-stamp-image-color".to_string(),
                "test-stamp-image-gray".to_string(),
                "This is a test stamp".to_string(),
            ))
            .await
            .unwrap();

        // テスト対象
        let repository = UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;

        let secret_key = "secret_key".to_string();
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let token = create_jwt(&test_user.id, iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        let path = format!("/challenges/{}/complete", test_challenge.id);

        let req = build_req_with_json_cookie(
            &path,
            Method::POST,
            format!("{{\"user_id\": \"{}\" }}", test_user.id).to_string(),
            &cookie_header,
        );

        create_challenge_routes(
            ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await,
            repository.clone(),
            "secret_key".to_string(),
        )
        .oneshot(req)
        .await
        .unwrap();

        let result = repository
            .query_user_completed_challenges(test_user.id)
            .await
            .unwrap();

        assert_eq!(result, vec![test_challenge.id])
    }

    #[tokio::test]
    async fn should_get_completed_challenges() {
        // ユーザーの作成
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let test_user = user_repository
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();

        // チャレンジの作成
        let challenge_repository = ChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let test_challenge = challenge_repository
            .create(CreateChallenge::new(
                "Test Challenge".to_string(),
                "This is a test challenge".to_string(),
                "test_id".to_string(),
                35.6895,
                139.6917,
                "Test Stamp".to_string(),
                "test-stamp-image-color".to_string(),
                "test-stamp-image-gray".to_string(),
                "This is a test stamp".to_string(),
            ))
            .await
            .unwrap();

        // クエスト参加を保存する
        let userchallenge_repository =
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let _ = userchallenge_repository
            .save_challenge_complete_event(test_user.id.clone(), test_challenge.id.clone())
            .await;

        // 認証のためにトークン作成
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let secret_key = "secret-key".to_string();
        let token = create_jwt(&test_user.id.clone(), iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        // テスト対象
        let userquest_repository = UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let req = build_req_with_cookie("/me/completed_challenges", Method::GET, &cookie_header);
        let res =
            create_user_info_routes(userquest_repository, userchallenge_repository, secret_key)
                .oneshot(req)
                .await
                .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let challenge_ids: Vec<String> = serde_json::from_str(&body).expect(&format!(
            "cannot convert Vec<String> instance. body {}",
            body
        ));
        assert_eq!(vec![test_challenge.id.clone()], challenge_ids);
    }

    #[tokio::test]
    async fn should_return_empty_vec_when_zero_completed_challenge() {
        // ユーザーの作成
        let user_repository = UserRepositoryForDb::with_url(DB_URL_FOR_TEST)
            .await
            .unwrap();
        let test_user = user_repository
            .register(RegisterUser::new(
                "test_user".to_string(),
                "test_email".to_string(),
                "test_password".to_string(),
            ))
            .await
            .unwrap();

        // 認証のためにトークン作成
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let secret_key = "secret-key".to_string();
        let token = create_jwt(&test_user.id.clone(), iat, &exp, &secret_key);
        let cookie_header = format!("session_token={}", token);

        // テスト対象
        let userquest_repository = UserQuestRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let userchallenge_repository =
            UserChallengeRepositoryForDb::with_url(DB_URL_FOR_TEST).await;
        let req = build_req_with_cookie("/me/completed_challenges", Method::GET, &cookie_header);
        let res =
            create_user_info_routes(userquest_repository, userchallenge_repository, secret_key)
                .oneshot(req)
                .await
                .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let quest_ids: Vec<String> = serde_json::from_str(&body).expect(&format!(
            "cannot convert Vec<String> instance. body {}",
            body
        ));
        assert_eq!(Vec::<String>::new(), quest_ids);
    }
}
