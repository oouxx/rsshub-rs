use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{
    http::fetch_json,
    response::{ErrorResponse, RssResponse},
};
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct V2exTopic {
    title: String,
    url: String,
    content: Option<String>,
    replies: u64,
    member: V2exMember,
    node: Option<V2exNode>,
    created: u64,
}

#[derive(Deserialize, Debug)]
struct V2exMember {
    username: String,
}

#[derive(Deserialize, Debug)]
struct V2exNode {
    title: String,
    name: String,
}

pub async fn latest_topics() -> impl IntoResponse {
    match fetch_v2ex("latest").await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

pub async fn hot_topics() -> impl IntoResponse {
    match fetch_v2ex("hot").await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error: {e}")).into_response(),
    }
}

async fn fetch_v2ex(endpoint: &str) -> anyhow::Result<String> {
    let url = format!("https://www.v2ex.com/api/v2/topics/{}", endpoint);

    // V2EX API v2 returns wrapped result
    #[derive(Deserialize)]
    struct V2exResponse {
        result: Vec<V2exTopic>,
    }

    let resp: V2exResponse = fetch_json(&url).await?;
    let topics = resp.result;

    let items: Vec<FeedItem> = topics
        .into_iter()
        .map(|topic| {
            let mut desc_parts = Vec::new();

            if let Some(node) = &topic.node {
                desc_parts.push(format!(
                    "📂 节点: <a href=\"https://www.v2ex.com/go/{}\">{}</a>",
                    node.name, node.title
                ));
            }
            desc_parts.push(format!("💬 回复数: {}", topic.replies));

            if let Some(content) = &topic.content {
                let preview: String = content.chars().take(500).collect();
                if !preview.is_empty() {
                    desc_parts.push(preview);
                }
            }

            let pub_date = chrono::DateTime::from_timestamp(topic.created as i64, 0);

            let categories = topic.node.map(|n| vec![n.title]).unwrap_or_default();

            FeedItem {
                title: topic.title,
                link: topic.url.clone(),
                description: desc_parts.join("<br/>"),
                author: Some(topic.member.username),
                pub_date,
                guid: Some(topic.url),
                categories,
            }
        })
        .collect();

    let (title, desc) = match endpoint {
        "latest" => ("V2EX 最新主题", "V2EX 最新发布的主题"),
        "hot" => ("V2EX 热门主题", "V2EX 今日热门主题"),
        _ => ("V2EX 主题", "V2EX 主题"),
    };

    let meta = FeedMeta {
        title: title.to_string(),
        link: "https://www.v2ex.com".to_string(),
        description: desc.to_string(),
        language: Some("zh-CN".to_string()),
    };

    build_rss(meta, items)
}
