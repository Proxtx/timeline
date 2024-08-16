use {
    web_sys::wasm_bindgen::JsValue,
    leptos::wasm_bindgen::JsCast,
    reqwest::Client,
    types::api::APIResult,
    url::{ParseError, Url},
    web_sys::js_sys::Function,
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
    Url::parse(&leptos::window().origin())?.join(path)
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
