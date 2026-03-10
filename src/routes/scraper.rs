use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{
    http::fetch_html,
    response::{ErrorResponse, RssResponse},
};
use axum::{extract::Query, response::IntoResponse};
use scraper::{Html, Selector};
use serde::Deserialize;

/// Generic scraper that extracts RSS from any page using CSS selectors
/// Query params:
///   url        - page URL to scrape (required)
///   item       - CSS selector for each item/article (required)
///   title      - CSS selector for title within each item (required)
///   link       - CSS selector for link within each item (optional, uses title's href)
///   desc       - CSS selector for description within each item
///   author     - CSS selector for author within each item
///   feed_title - Override feed title
#[derive(Deserialize, Debug)]
pub struct ScrapeParams {
    url: String,
    item: String,
    title: String,
    link: Option<String>,
    desc: Option<String>,
    author: Option<String>,
    feed_title: Option<String>,
}

pub async fn scrape(Query(params): Query<ScrapeParams>) -> impl IntoResponse {
    match do_scrape(params).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Scrape failed: {e}")).into_response(),
    }
}

async fn do_scrape(params: ScrapeParams) -> anyhow::Result<String> {
    let html = fetch_html(&params.url).await?;
    let document = Html::parse_document(&html);

    let item_sel = Selector::parse(&params.item)
        .map_err(|_| anyhow::anyhow!("Invalid item selector: {}", params.item))?;
    let title_sel = Selector::parse(&params.title)
        .map_err(|_| anyhow::anyhow!("Invalid title selector: {}", params.title))?;

    let link_sel = params
        .link
        .as_ref()
        .map(|s| Selector::parse(s).map_err(|_| anyhow::anyhow!("Invalid link selector")))
        .transpose()?;

    let desc_sel = params
        .desc
        .as_ref()
        .map(|s| Selector::parse(s).map_err(|_| anyhow::anyhow!("Invalid desc selector")))
        .transpose()?;

    let author_sel = params
        .author
        .as_ref()
        .map(|s| Selector::parse(s).map_err(|_| anyhow::anyhow!("Invalid author selector")))
        .transpose()?;

    // Get base URL for resolving relative links
    let base_url = {
        let parsed = url::Url::parse(&params.url)?;
        format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""))
    };

    let mut items = Vec::new();

    for el in document.select(&item_sel) {
        let title_el = match el.select(&title_sel).next() {
            Some(t) => t,
            None => continue,
        };

        let title = title_el.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }

        // Try to get link from title's href, or from link selector
        let link = if let Some(lsel) = &link_sel {
            el.select(lsel)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|h| resolve_url(h, &base_url))
                .unwrap_or_default()
        } else {
            title_el
                .value()
                .attr("href")
                .map(|h| resolve_url(h, &base_url))
                .unwrap_or_else(|| params.url.clone())
        };

        let description = desc_sel
            .as_ref()
            .and_then(|sel| el.select(sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let author = author_sel
            .as_ref()
            .and_then(|sel| el.select(sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string());

        items.push(FeedItem {
            title,
            link: link.clone(),
            description,
            author,
            pub_date: None,
            guid: Some(link),
            categories: vec![],
        });
    }

    // Get page title from <title> tag if not overridden
    let page_title = params.feed_title.unwrap_or_else(|| {
        let title_sel = Selector::parse("title").unwrap();
        document
            .select(&title_sel)
            .next()
            .map(|t| t.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| params.url.clone())
    });

    let meta = FeedMeta {
        title: page_title,
        link: params.url.clone(),
        description: format!("Scraped RSS via RSS Forge from {}", params.url),
        language: None,
    };

    build_rss(meta, items)
}

fn resolve_url(href: &str, base: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with('/') {
        format!("{}{}", base, href)
    } else {
        format!("{}/{}", base, href)
    }
}
