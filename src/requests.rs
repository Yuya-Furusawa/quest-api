use async_session::SessionStore;
use async_sqlx_session::PostgresSessionStore;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
    response::Redirect,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
struct UserContext {
    user_id: String,
}

#[async_trait]
impl<B> FromRequest<B> for UserContext
where
    B: Send,
{
    type Rejection = Redirect;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let redirect = || Redirect::to("/login");

        let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
        let store = PostgresSessionStore::new(&database_url)
            .await
            .map_err(|_| redirect())?;
        let cookies = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap()
            .ok_or(redirect())?;
        let session_str = cookies.get("session_id").ok_or(redirect())?;
        let session = store
            .load_session(session_str.to_string())
            .await
            .map_err(|_| redirect())?;
        let session = session.ok_or(redirect());
        let context = UserContext {
            user_id: session.unwrap().get("session_id").unwrap(),
        };
        Ok(context)
    }
}
