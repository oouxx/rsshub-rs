use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;
use crate::feeds::{build_rss, FeedItem, FeedMeta};
use crate::utils::{http::HTTP_CLIENT, response::{RssResponse, ErrorResponse}};

const API_BASE: &str = "https://baoer-api.xuangubao.com.cn/api/v6/message/newsflash";
// subj_ids 说明（从抓包观察）：
//   9   = 7x24快讯（通用）
//   10  = 公告
//   723 = 财经要闻
//   35  = 港股
//   469 = 美股
//   821 = 期货
const DEFAULT_SUBJ_IDS: &str = "9,10,723,35,469,821";

#[derive(Deserialize)]
pub struct LiveQuery {
    /// 自定义订阅分类，逗号分隔，默认全部
    subj_ids: Option<String>,
    /// 每次拉取条数，默认 30
    limit: Option<u32>,
    /// 分页游标（next_cursor），拉取更早内容
    cursor: Option<i64>,
}

// ── JSON 结构 ───────────────────────────────────────────────

#[derive(Deserialize)]
struct ApiResponse {
    data: ApiData,
}

#[derive(Deserialize)]
struct ApiData {
    messages: Vec<NewsFlash>,
    next_cursor: Option<i64>,
}

#[derive(Deserialize)]
struct NewsFlash {
    id: u64,
    title: String,
    summary: Option<String>,
    stocks: Vec<Stock>,
    bkj_infos: Vec<BkjInfo>,
    created_at: i64,
    route: Option<String>,
}

#[derive(Deserialize)]
struct Stock {
    name: String,
    symbol: String,
}

#[derive(Deserialize)]
struct BkjInfo {
    name: String,
}

// ── Handler ─────────────────────────────────────────────────

pub async fn live(Query(q): Query<LiveQuery>) -> impl IntoResponse {
    match fetch_newsflash(q).await {
        Ok(rss) => RssResponse(rss).into_response(),
        Err(e) => ErrorResponse(format!("选股通快讯抓取失败: {e}")).into_response(),
    }
}

async fn fetch_newsflash(q: LiveQuery) -> anyhow::Result<String> {
    let subj_ids = q.subj_ids.as_deref().unwrap_or(DEFAULT_SUBJ_IDS);
    let limit = q.limit.unwrap_or(30).min(50);

    let mut url = format!(
        "{}?limit={}&subj_ids={}&platform=pcweb",
        API_BASE, limit, subj_ids
    );
    if let Some(cursor) = q.cursor {
        url.push_str(&format!("&cursor={}", cursor));
    }

    let resp = HTTP_CLIENT
        .get(&url)
        .header("Referer", "https://xuangutong.com.cn/live")
        .header("Origin", "https://xuangutong.com.cn")
        .send()
        .await?;

    let api: ApiResponse = resp.json().await?;

    let items: Vec<FeedItem> = api.data.messages
        .into_iter()
        .map(|msg| {
            let link = msg.route.unwrap_or_else(|| {
                format!("https://xuangubao.com.cn/article/{}", msg.id)
            });

            let mut desc_parts: Vec<String> = Vec::new();

            if let Some(summary) = &msg.summary {
                if !summary.is_empty() {
                    desc_parts.push(summary.clone());
                }
            }

            if !msg.stocks.is_empty() {
                let stocks_str = msg.stocks.iter()
                    .map(|s| format!("{} ({})", s.name, s.symbol))
                    .collect::<Vec<_>>()
                    .join("、");
                desc_parts.push(format!("📈 相关股票：{}", stocks_str));
            }

            if !msg.bkj_infos.is_empty() {
                let tags = msg.bkj_infos.iter()
                    .map(|b| b.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" · ");
                desc_parts.push(format!("🏷 {}", tags));
            }

            let pub_date = chrono::DateTime::from_timestamp(msg.created_at, 0)
                .map(|dt| dt.into());

            let categories = msg.bkj_infos.iter()
                .map(|b| b.name.clone())
                .collect();

            FeedItem {
                title: msg.title,
                link: link.clone(),
                description: desc_parts.join("<br/>"),
                author: None,
                pub_date,
                guid: Some(format!("xuangutong-{}", msg.id)),
                categories,
            }
        })
        .collect();

    let cursor_hint = api.data.next_cursor
        .map(|c| format!(" (next_cursor={})", c))
        .unwrap_or_default();

    let meta = FeedMeta {
        title: "选股通 7x24 快讯".to_string(),
        link: "https://xuangutong.com.cn/live".to_string(),
        description: format!("选股通 7x24 小时财经快讯{}", cursor_hint),
        language: Some("zh-CN".to_string()),
    };

    build_rss(meta, items)
}
