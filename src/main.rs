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

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/health", get(|| async { "OK" }))
        // GitHub routes
        .route("/github/trending", get(routes::github::trending))
        .route(
            "/github/trending/:lang",
            get(routes::github::trending_by_lang),
        )
        // Hacker News routes
        .route("/hackernews/best", get(routes::hackernews::best))
        .route("/hackernews/new", get(routes::hackernews::new_stories))
        .route("/hackernews/show", get(routes::hackernews::show))
        .route("/hackernews/ask", get(routes::hackernews::ask))
        // Reddit routes
        .route("/reddit/:subreddit", get(routes::reddit::subreddit))
        .route("/reddit/:subreddit/top", get(routes::reddit::subreddit_top))
        // V2EX routes
        .route("/v2ex/topics/latest", get(routes::v2ex::latest_topics))
        .route("/v2ex/topics/hot", get(routes::v2ex::hot_topics))
        // xuangubao
        .route("/xuangutao/live", get(routes::xuangubao::live))
        // Generic scraper route
        .route("/scrape", get(routes::scraper::scrape))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .fallback(not_found);

    let addr = "0.0.0.0:3000";
    tracing::info!("🚀 RSS Forge listening on http://{}", addr);
    tracing::info!("📡 Available routes:");
    tracing::info!("   /github/trending");
    tracing::info!("   /github/trending/:lang");
    tracing::info!("   /hackernews/best");
    tracing::info!("   /hackernews/new");
    tracing::info!("   /hackernews/show");
    tracing::info!("   /hackernews/ask");
    tracing::info!("   /reddit/:subreddit");
    tracing::info!("   /v2ex/topics/latest");
    tracing::info!("   /v2ex/topics/hot");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<String> {
    Html(include_str!("../static/index.html").to_string())
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Route not found. Visit / for available routes.",
    )
}
