use {
    rocket::{http::CookieJar, State},
    types::api::{APIError, APIResult},
};

use crate::config::Config;

pub fn auth(cookies: &CookieJar<'_>, config: &State<Config>) -> APIResult<()> {
    match cookies.get("pwd") {
        Some(pwd) => {
            if pwd.value() != config.password {
                Err(APIError::AuthenticationError)
            } else {
                Ok(())
            }
        }
        None => Err(APIError::AuthenticationError),
    }
}
