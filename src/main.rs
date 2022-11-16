mod handlers;
mod repositories;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use std::{
    net::SocketAddr,
    sync::Arc,
};

use crate::handlers::quest::{create_quest, find_quest, all_quests, update_quest, delete_quest};
use crate::repositories::quest::{QuestRepository, QuestRepositoryForMemory};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = create_app(QuestRepositoryForMemory::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: QuestRepository>(
    quest_repository: T,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route(
            "/quests",
            post(create_quest::<T>)
                .get(all_quests::<T>)
        )
        .route(
            "/quests/:id",
            get(find_quest::<T>)
                .patch(update_quest::<T>)
                .delete(delete_quest::<T>)
        )
        .layer(Extension(Arc::new(quest_repository)))
}

async fn root() -> &'static str {
    "Hello World!"
}
