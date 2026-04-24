//! Bearer-token request guard. The main timeline server sends
//! `Authorization: Bearer <token>` on every proxied call; the SDK rejects
//! anything without a matching token (the `Plugin::token` from `config.toml`).

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;

use crate::launch::PluginState;

pub struct AuthedClient;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthedClient {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state: &State<PluginState> = match req.guard::<&State<PluginState>>().await {
            Outcome::Success(s) => s,
            _ => return Outcome::Error((Status::InternalServerError, AuthError::StateMissing)),
        };

        let expected = &state.token;

        let Some(header) = req.headers().get_one("Authorization") else {
            return Outcome::Error((Status::Unauthorized, AuthError::Missing));
        };

        let Some(token) = header.strip_prefix("Bearer ") else {
            return Outcome::Error((Status::Unauthorized, AuthError::WrongScheme));
        };

        if constant_time_eq(token.as_bytes(), expected.as_bytes()) {
            Outcome::Success(AuthedClient)
        } else {
            Outcome::Error((Status::Unauthorized, AuthError::BadToken))
        }
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    WrongScheme,
    BadToken,
    StateMissing,
}
