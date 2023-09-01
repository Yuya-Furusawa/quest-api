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
