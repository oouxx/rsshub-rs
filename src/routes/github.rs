use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{
    http::fetch_html,
    response::{ErrorResponse, RssResponse},
};
use axum::{extract::Path, response::IntoResponse};
use scraper::{Html, Selector};

pub async fn trending() -> impl IntoResponse {
    match fetch_trending(None).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("Failed to fetch GitHub trending: {e}")).into_response(),
    }
}

pub async fn trending_by_lang(Path(lang): Path<String>) -> impl IntoResponse {
    match fetch_trending(Some(&lang)).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => {
            ErrorResponse(format!("Failed to fetch GitHub trending ({lang}): {e}")).into_response()
        }
    }
}

async fn fetch_trending(lang: Option<&str>) -> anyhow::Result<String> {
    let url = match lang {
        Some(l) => format!("https://github.com/trending/{}", l),
        None => "https://github.com/trending".to_string(),
    };

    let html = fetch_html(&url).await?;
    let document = Html::parse_document(&html);

    let repo_sel = Selector::parse("article.Box-row").unwrap();
    let name_sel = Selector::parse("h2.h3 a").unwrap();
    let desc_sel = Selector::parse("p.col-9").unwrap();
    let lang_sel = Selector::parse("span[itemprop='programmingLanguage']").unwrap();
    let stars_sel = Selector::parse("a.Link--muted").unwrap();
    let stars_today_sel = Selector::parse("span.d-inline-block.float-sm-right").unwrap();

    let mut items = Vec::new();

    for repo in document.select(&repo_sel) {
        let name_el = match repo.select(&name_sel).next() {
            Some(el) => el,
            None => continue,
        };

        let href = name_el.value().attr("href").unwrap_or("");
        let repo_name = name_el
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .replace('\n', "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("/");
        let link = format!("https://github.com{}", href);

        let description = repo
            .select(&desc_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let language = repo
            .select(&lang_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());

        let stars: Vec<String> = repo
            .select(&stars_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .collect();

        let stars_today = repo
            .select(&stars_today_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let mut desc_parts = vec![description.clone()];
        if let Some(lang) = &language {
            desc_parts.push(format!("Language: {}", lang));
        }
        if let Some(s) = stars.first() {
            desc_parts.push(format!("Stars: {}", s.trim()));
        }
        if !stars_today.is_empty() {
            desc_parts.push(format!("Stars today: {}", stars_today));
        }

        items.push(FeedItem {
            title: repo_name.clone(),
            link: link.clone(),
            description: desc_parts.join(" | "),
            author: None,
            pub_date: None,
            guid: Some(link),
            categories: language.into_iter().collect(),
        });
    }

    let lang_title = lang.map(|l| format!(" - {}", l)).unwrap_or_default();
    let meta = FeedMeta {
        title: format!("GitHub Trending{}", lang_title),
        link: url,
        description: format!("GitHub trending repositories{}", lang_title),
        language: Some("en".to_string()),
    };

    build_rss(meta, items)
}
