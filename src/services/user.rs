use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    iat: i64,
    exp: i64,
}

pub fn create_jwt(user_id: &String, iat: i64, exp: &i64) -> String {
    dotenv().ok();
    let secret_key = &env::var("JWT_SECRET_KEY").expect("undefined [JWT_SECRET_KEY]");

    let my_claims = Claims {
        user_id: user_id.clone(),
        iat: iat,
        exp: *exp,
    };

    encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .unwrap()
}

pub fn decode_jwt(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    dotenv().ok();
    let secret_key = &env::var("JWT_SECRET_KEY").expect("undefined [JWT_SECRET_KEY]");
    decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )
}
