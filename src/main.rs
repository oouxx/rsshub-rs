mod docs;
mod feeds;
mod routes;
mod utils;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rss_forge=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new();
    let app = droute!(app, "root", "/", get(index_handler), "首页");
    let app = droute!(app, "health", "/health", get(|| async { "OK" }), "健康检查");
    let app = droute!(
        app,
        "github",
        "/github/trending",
        get(routes::github::trending),
        "GitHub 趋势"
    );
    let app = droute!(
        app,
        "github",
        "/github/trending/:lang",
        get(routes::github::trending_by_lang),
        "GitHub 趋势 (按语言)"
    );
    let app = droute!(
        app,
        "hackernews",
        "/hackernews/best",
        get(routes::hackernews::best),
        "Hacker News 最佳"
    );
    let app = droute!(
        app,
        "hackernews",
        "/hackernews/new",
        get(routes::hackernews::new_stories),
        "Hacker News 最新"
    );
    let app = droute!(
        app,
        "hackernews",
        "/hackernews/show",
        get(routes::hackernews::show),
        "Hacker News 展示"
    );
    let app = droute!(
        app,
        "hackernews",
        "/hackernews/ask",
        get(routes::hackernews::ask),
        "Hacker News Ask HN"
    );
    let app = droute!(
        app,
        "reddit",
        "/reddit/:subreddit",
        get(routes::reddit::subreddit),
        "Reddit 子版块"
    );
    let app = droute!(
        app,
        "reddit",
        "/reddit/:subreddit/top",
        get(routes::reddit::subreddit_top),
        "Reddit 子版块 Top"
    );
    // V2EX routes
    let app = droute!(
        app,
        "v2ex",
        "/v2ex/topics/latest",
        get(routes::v2ex::latest_topics),
        "V2EX 最新话题"
    );
    let app = droute!(
        app,
        "v2ex",
        "/v2ex/topics/hot",
        get(routes::v2ex::hot_topics),
        "V2EX 热门话题"
    );
    // xuangubao
    let app = droute!(
        app,
        "xuangubao",
        "/xuangubao/live",
        get(routes::xuangubao::live),
        "Xuangubao 直播"
    );
    let app = droute!(
        app,
        "scrape",
        "/scrape",
        get(routes::scraper::scrape),
        "Generic Scraper"
    );
    // 中间件
    let app = app
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .fallback(not_found);

    let addr = "0.0.0.0:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Route not found. Visit / for available routes.",
    )
}

use crate::docs::routes::ROUTES;
use askama::Template;
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: &'a Vec<crate::docs::routes::RouteDoc>,
}

#[axum::debug_handler]
async fn index_handler() -> impl axum::response::IntoResponse {
    let routes = ROUTES.lock().unwrap();
    let template = IndexTemplate { routes: &routes };

    // 将 askama::Error 转换成 HTTP 500
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", e),
        )
            .into_response(),
    }
}
