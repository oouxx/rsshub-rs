use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RSSForge/1.0; +https://github.com/rss-forge)")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client")
});

pub async fn fetch_html(url: &str) -> Result<String> {
    let resp = HTTP_CLIENT
        .get(url)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await?;

    Ok(resp.text().await?)
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp = HTTP_CLIENT
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;

    Ok(resp.json::<T>().await?)
}
