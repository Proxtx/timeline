use reqwest::Client;
use types::api::APIResult;
use url::Url;

pub async fn api_request<T, V>(endpoint: &str, request: &V) -> APIResult<T>
where
T: serde::de::DeserializeOwned,
V: serde::Serialize,
{
    let client = Client::new();
    let url = Url::parse(&leptos::window().origin()).unwrap().join(&format!("/api{}", endpoint)).unwrap();
    serde_json::from_str(&client.post(url).body(serde_json::to_string(request)?).send().await?.text().await?)?
}