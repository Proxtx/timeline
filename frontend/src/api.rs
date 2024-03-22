use std::error::Error;

use leptos::logging;
use reqwest::Client;
use types::api::{APIError, APIResult};
use url::{ParseError, Url};
use leptos::wasm_bindgen::JsCast;
use web_sys::js_sys::Function;
use crate::wasm_bindgen::JsValue;

pub async fn api_request<T, V>(endpoint: &str, request: &V) -> APIResult<T>
where
T: serde::de::DeserializeOwned,
V: serde::Serialize,
{
    let client = Client::new();
    let url = relative_url(&format!("/api{}", endpoint)).unwrap();
    let res = serde_json::from_str::<APIResult<T>>(&client.post(url).body(serde_json::to_string(request)?).send().await?.text().await?)?;
    match res {
        Ok(v) => {
            APIResult::Ok(v)
        }
        Err(e) => {
            logging::log!("{:?}", e);
            match e {
                APIError::AuthenticationError => {
                    leptos::window().location().set_pathname("/login").unwrap();
                    Err(e)
                }
                e => {
                    Err(e)
                } 
            }
        }
    }
}

pub fn relative_url(path: &str) -> Result<Url, ParseError> {
    Url::parse(&leptos::window().origin())?.join(path)
}

pub fn encode_url_component(data: &str) -> String {
    leptos::window().get("encodeURIComponent").unwrap().dyn_into::<Function>().unwrap().call1(&JsValue::null(), &JsValue::from_str(data)).unwrap().as_string().unwrap()
}