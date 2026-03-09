use axum::{extract::Path, response::IntoResponse};
use serde::Deserialize;
use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{http::fetch_json, response::{RssResponse, ErrorResponse}};

#[derive(Deserialize, Debug)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Deserialize, Debug)]
struct RedditListingData {
    children: Vec<RedditPost>,
}

#[derive(Deserialize, Debug)]
struct RedditPost {
    data: RedditPostData,
}

#[derive(Deserialize, Debug)]
struct RedditPostData {
    title: String,
    url: String,
    permalink: String,
    author: String,
    score: i64,
    num_comments: u64,
    selftext: Option<String>,
    created_utc: f64,
    subreddit: String,
    thumbnail: Option<String>,
    is_self: bool,
    flair_text: Option<String>,
}

pub async fn subreddit(Path(sub): Path<String>) -> impl IntoResponse {
    match fetch_subreddit(&sub, "hot").await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error fetching r/{sub}: {e}")).into_response(),
    }
}

pub async fn subreddit_top(Path(sub): Path<String>) -> impl IntoResponse {
    match fetch_subreddit(&sub, "top").await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Error fetching r/{sub} top: {e}")).into_response(),
    }
}

async fn fetch_subreddit(sub: &str, sort: &str) -> anyhow::Result<String> {
    let url = format!("https://www.reddit.com/r/{}/{}.json?limit=25", sub, sort);
    let listing: RedditListing = fetch_json(&url).await?;

    let items: Vec<FeedItem> = listing
        .data
        .children
        .into_iter()
        .map(|post| {
            let d = post.data;
            let permalink = format!("https://reddit.com{}", d.permalink);

            let mut desc_parts = vec![
                format!("⬆️ {} | 💬 {} comments", d.score, d.num_comments),
            ];

            if let Some(flair) = &d.flair_text {
                if !flair.is_empty() {
                    desc_parts.push(format!("[{}]", flair));
                }
            }

            if d.is_self {
                if let Some(text) = &d.selftext {
                    let preview: String = text.chars().take(400).collect();
                    if !preview.is_empty() {
                        desc_parts.push(preview);
                    }
                }
            } else if let Some(thumb) = &d.thumbnail {
                if thumb.starts_with("http") {
                    desc_parts.push(format!("<img src=\"{}\" />", thumb));
                }
            }

            desc_parts.push(format!("🔗 <a href=\"{}\">Comments on Reddit</a>", permalink));

            let pub_date = chrono::DateTime::from_timestamp(d.created_utc as i64, 0)
                .map(|dt| dt.into());

            FeedItem {
                title: d.title,
                link: if d.is_self { permalink.clone() } else { d.url },
                description: desc_parts.join("<br/>"),
                author: Some(format!("u/{}", d.author)),
                pub_date,
                guid: Some(permalink),
                categories: vec![d.subreddit],
            }
        })
        .collect();

    let meta = FeedMeta {
        title: format!("r/{} - {}", sub, sort),
        link: format!("https://www.reddit.com/r/{}", sub),
        description: format!("Reddit r/{} {} posts via RSS Forge", sub, sort),
        language: Some("en".to_string()),
    };

    build_rss(meta, items)
}
