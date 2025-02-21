use std::{fmt::Display, fs::OpenOptions};

use serde::Deserialize;
use rss::{Channel, ChannelBuilder, ItemBuilder};

/// # Safety
///
/// `*const u16` はnull終端UTF16文字列のポインタ
/// `*mut u16` はUTF16文字列バッファのポインタ
#[no_mangle]
pub unsafe extern "C" fn generate(json: *const u16, out: *const u16, error: *mut u16) -> bool {
    let json = string_from_ptr(json);
    let out = string_from_ptr(out);
    let feed = match serde_json::from_str::<FeedJson>(&json) {
        Ok(mut feed) => {
            feed.sort_items();
            feed
        },
        Err(e) => {
            set_error(error, e);
            return false;
        },
    };

    let channel = Channel::from(feed);

    let f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out);
    let writer = match f {
        Ok(f) => f,
        Err(e) => {
            set_error(error, e);
            return false;
        },
    };

    match channel.pretty_write_to(writer, b' ', 2) {
        Ok(f) => {
            drop(f);
        },
        Err(e) => {
            set_error(error, e);
            return false
        },
    }

    true
}

unsafe fn set_error<S: Display>(error: *mut u16, msg: S) {
    let utf16 = msg.to_string().encode_utf16().collect::<Vec<_>>();
    let count = utf16.len();
    let src = utf16.as_ptr() as _;
    error.copy_from_nonoverlapping(src, count);
}
unsafe fn string_from_ptr(p: *const u16) -> String {
    let len = (0..).take_while(|&i| *p.offset(i) != 0).count();
    let wide = std::slice::from_raw_parts(p, len);

    String::from_utf16_lossy(wide)
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


#[cfg(test)]
mod test {
    fn to_utf16(str: &str) -> Vec<u16> {
        str.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn test_generate() {
        let mut error = vec![0u16; 256];
        let out = to_utf16("target/test.xml");
        let json = to_utf16(r#"{
    "title": "ほげほげ",
    "link": "https://example.com",
    "pubDate": "2025/02/03",
    "description": "てすと",
    "items": [
        {
            "title": "あいてむ2",
            "link": "https://example.com/item2",
            "pubDate": "2025/02/02",
            "pubSec": 791769600,
            "description": "てすと2"
        },
        {
            "title": "あいてむ1",
            "link": "https://example.com/item1",
            "pubDate": "2025/02/03",
            "pubSec": 791856000,
            "description": "てすと1"
        }
    ]
}"#);
        assert!(
            unsafe {
                super::generate(json.as_ptr(), out.as_ptr(), error.as_mut_ptr() as _)
            }
        );
    }
}
