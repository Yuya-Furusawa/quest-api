use async_session::{Session, SessionStore};
use async_sqlx_session::PostgresSessionStore;
use dotenv::dotenv;
use std::{env, time::Duration};

use crate::repositories::user::UserEntity;

pub async fn create_session(user: &UserEntity) -> SessionToken {
    dotenv().ok();
    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    let store = PostgresSessionStore::new(&database_url).await.unwrap();

    let mut session = Session::new();
    session.insert("session_id", (user.id).clone()).unwrap();
    session.expire_in(Duration::from_secs(60));

    let cookie = store.store_session(session).await.unwrap().unwrap();

    SessionToken::new(&cookie)
}

pub struct SessionToken {
    token: String,
    max_age: usize,
}

impl SessionToken {
    pub fn new(token: &str) -> SessionToken {
        SessionToken {
            token: token.to_string(),
            max_age: 60,
        }
    }

    pub fn cookie(&self) -> String {
        format!(
            "{}={}; Max-Age={}; Path=/; HttpOnly",
            "session_id", self.token, self.max_age
        )
    }
}
