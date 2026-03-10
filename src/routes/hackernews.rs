use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{
    http::fetch_json,
    response::{ErrorResponse, RssResponse},
};
use axum::response::IntoResponse;
use futures::future::join_all;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HnItem {
    id: u64,
    title: Option<String>,
    url: Option<String>,
    by: Option<String>,
    score: Option<u64>,
    descendants: Option<u64>,
    time: Option<i64>,
    text: Option<String>,
    #[serde(rename = "type")]
    item_type: Option<String>,
}

pub async fn best() -> impl IntoResponse {
    match fetch_hn_stories("beststories", "HN Best Stories", 30).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

pub async fn new_stories() -> impl IntoResponse {
    match fetch_hn_stories("newstories", "HN New Stories", 30).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

pub async fn show() -> impl IntoResponse {
    match fetch_hn_stories("showstories", "HN Show HN", 30).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

pub async fn ask() -> impl IntoResponse {
    match fetch_hn_stories("askstories", "HN Ask HN", 30).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

async fn fetch_hn_stories(endpoint: &str, title: &str, limit: usize) -> anyhow::Result<String> {
    let ids_url = format!("https://hacker-news.firebaseio.com/v0/{}.json", endpoint);
    let ids: Vec<u64> = fetch_json(&ids_url).await?;

    let ids_limited: Vec<u64> = ids.into_iter().take(limit).collect();

    let futures = ids_limited.iter().map(|id| {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
        async move { fetch_json::<HnItem>(&url).await }
    });

    let results = join_all(futures).await;

    let items: Vec<FeedItem> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|item| {
            let title = item.title?;
            let id = item.id;
            let hn_link = format!("https://news.ycombinator.com/item?id={}", id);
            let link = item.url.unwrap_or_else(|| hn_link.clone());

            let mut desc_parts = Vec::new();
            if let Some(score) = item.score {
                desc_parts.push(format!("⭐ {} points", score));
            }
            if let Some(comments) = item.descendants {
                desc_parts.push(format!("💬 {} comments", comments));
            }
            if let Some(text) = item.text {
                let clean = strip_html(&text);
                if !clean.is_empty() {
                    desc_parts.push(clean.chars().take(500).collect::<String>());
                }
            }
            desc_parts.push(format!("🔗 <a href=\"{}\">Comments</a>", hn_link));

            let pub_date = item
                .time
                .map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap_or_default());

            Some(FeedItem {
                title,
                link,
                description: desc_parts.join(" | "),
                author: item.by,
                pub_date,
                guid: Some(hn_link),
                categories: item.item_type.into_iter().collect(),
            })
        })
        .collect();

    let meta = FeedMeta {
        title: title.to_string(),
        link: "https://news.ycombinator.com".to_string(),
        description: format!("{} - RSS via RSS Forge", title),
        language: Some("en".to_string()),
    };

    build_rss(meta, items)
}

fn strip_html(s: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    let result = re.replace_all(s, " ");
    html_escape::decode_html_entities(&result)
        .trim()
        .to_string()
}
