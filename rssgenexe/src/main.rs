use std::{fs::OpenOptions, io::Read};

use serde::Deserialize;
use rss::{Channel, ChannelBuilder, ItemBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut args = std::env::args();
    if let (Some(json_path), Some(xml_path)) = (args.nth(1), args.next()) {
        let mut file = std::fs::File::open(json_path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let mut feed = serde_json::from_str::<FeedJson>(&json)?;
        feed.sort_items();

        let channel = Channel::from(feed);

        let writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(xml_path)?;

        let _f = channel.pretty_write_to(writer, b' ', 2)?;

        Ok(())
    } else {
        eprintln!("usage: rssgenexe input.json output.xml");
        Ok(())
    }
}



impl From<FeedJson> for Channel {
    fn from(feed: FeedJson) -> Self {
        let mut channel = ChannelBuilder::default()
            .title(&feed.title)
            .link(&feed.link)
            .description(&feed.description)
            .pub_date(Some(feed.pub_date))
            .build();
        let items = feed.items.into_iter()
            .map(|feed_item| {
                ItemBuilder::default()
                    .title(Some(feed_item.title))
                    .link(Some(feed_item.link))
                    .description(Some(feed_item.description))
                    .pub_date(Some(feed_item.pub_date))
                    .build()
            })
            .collect::<Vec<_>>();
        channel.set_items(items);
        channel
    }
}

#[derive(Debug, Deserialize)]
struct FeedJson {
    title: String,
    link: String,
    #[serde(rename(deserialize = "pubDate"))]
    pub_date: String,
    description: String,
    items: Vec<FeedItem>,
}

impl FeedJson {
    fn sort_items(&mut self) {
        self.items.sort_by(|a, b| b.compare(a));
    }
}

#[derive(Debug, Deserialize)]
struct FeedItem {
    title: String,
    description: String,
    link: String,
    #[serde(rename(deserialize = "pubDate"))]
    pub_date: String,
    #[serde(rename(deserialize = "pubSec"))]
    pub_sec: f64,
}
impl FeedItem {
    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.pub_sec as usize;
        let b = other.pub_sec as usize;
        a.cmp(&b)
    }
}


