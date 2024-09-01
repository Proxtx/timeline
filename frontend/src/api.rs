use {
    leptos::{use_context, wasm_bindgen::JsCast},
    reqwest::Client,
    types::api::{APIResult, TimelineHostname},
    url::{ParseError, Url},
    web_sys::{js_sys::Function, wasm_bindgen::JsValue},
};
pub async fn api_request<T, V>(endpoint: &str, request: &V) -> APIResult<T>
where
    T: serde::de::DeserializeOwned,
    V: serde::Serialize,
{
    let client = Client::new();
    let url = relative_url(&format!("/api{}", endpoint)).unwrap();
    serde_json::from_str::<APIResult<T>>(
        &client
            .post(url)
            .body(serde_json::to_string(request)?)
            .send()
            .await?
            .text()
            .await?,
    )?
}

pub fn relative_url(path: &str) -> Result<Url, ParseError> {
    let timeline_host: TimelineHostname = use_context().unwrap();
    Url::parse(&timeline_host.0)?.join(path)
}

#[allow(unused)]
pub fn encode_url_component(data: &str) -> String {
    leptos::window()
        .get("encodeURIComponent")
        .unwrap()
        .dyn_into::<Function>()
        .unwrap()
        .call1(&JsValue::null(), &JsValue::from_str(data))
        .unwrap()
        .as_string()
        .unwrap()
}
