use axum::{
    headers::HeaderMapExt,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::services::user::decode_jwt;

pub async fn auth_middleware<B>(
    secret_key: String,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if let Some(cookies) = req.headers().typed_get::<axum::headers::Cookie>() {
        if let Some(session_token) = cookies.get("session_token") {
            let decoded_token = decode_jwt(session_token, &secret_key).unwrap();
            req.extensions_mut().insert(decoded_token.claims.user_id);
            return Ok(next.run(req).await);
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::services::user::create_jwt;
    use axum::{
        http::{Request, StatusCode},
        middleware::from_fn,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use chrono::{Duration, Utc};
    use hyper::Body;
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_cookie() {
        let secret_key = "secret_key".to_string();
        let test_user_id = "test_user".to_string();
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::hours(8)).timestamp();
        let valid_session_token = create_jwt(&test_user_id, iat, &exp, &secret_key);

        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn(move |req, next| {
                auth_middleware(secret_key.clone(), req, next)
            }));

        let req = Request::builder()
            .header("cookie", format!("session_token={}", valid_session_token))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK)
    }
}
