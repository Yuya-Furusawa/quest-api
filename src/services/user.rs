use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    iat: i64,
    exp: i64,
}

pub fn create_jwt(user_id: &String, iat: i64, exp: &i64, secret_key: &String) -> String {
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
    .expect("Failed to encode token. Likely wrong secret keys")
}

pub fn decode_jwt(
    jwt: &str,
    secret_key: &String,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )
}
