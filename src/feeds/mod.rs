use anyhow::Result;
use chrono::{DateTime, Utc};
use rss::{ChannelBuilder, GuidBuilder, Item, ItemBuilder};

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub author: Option<String>,
    pub pub_date: Option<DateTime<Utc>>,
    pub guid: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug)]
pub struct FeedMeta {
    pub title: String,
    pub link: String,
    pub description: String,
    pub language: Option<String>,
}

pub fn build_rss(meta: FeedMeta, items: Vec<FeedItem>) -> Result<String> {
    let rss_items: Vec<Item> = items
        .into_iter()
        .map(|item| {
            let mut builder = ItemBuilder::default();
            builder.title(Some(item.title));
            builder.link(Some(item.link.clone()));
            builder.description(Some(item.description));

            if let Some(author) = item.author {
                builder.author(Some(author));
            }

            if let Some(date) = item.pub_date {
                builder.pub_date(Some(date.to_rfc2822()));
            }

            let guid_val = item.guid.unwrap_or(item.link);
            let guid = GuidBuilder::default()
                .value(guid_val)
                .permalink(true)
                .build();
            builder.guid(Some(guid));

            if !item.categories.is_empty() {
                let cats: Vec<rss::Category> = item
                    .categories
                    .iter()
                    .map(|c| rss::CategoryBuilder::default().name(c.clone()).build())
                    .collect();
                builder.categories(cats);
            }

            builder.build()
        })
        .collect();

    let mut channel_builder = ChannelBuilder::default();
    channel_builder
        .title(meta.title)
        .link(meta.link)
        .description(meta.description)
        .items(rss_items);

    if let Some(lang) = meta.language {
        channel_builder.language(Some(lang));
    }

    let channel = channel_builder.build();
    Ok(channel.to_string())
}
