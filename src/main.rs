#![deny(clippy::all)]

use std::{collections::HashSet, fmt};

#[derive(Debug, Hash, Eq, PartialEq)]
struct Node {
    id: String,
    group: i32,
}

type Nodes = HashSet<Node>;

#[derive(Debug, Hash, Eq, PartialEq)]
struct Link {
    source: String,
    target: String,
    value: i32,
}

type Links = HashSet<Link>;

#[derive(Debug)]
struct Db {
    nodes: Nodes,
    links: Links,
}

const MAX_DEPTH: i32 = 10;
const SELECTOR: scraper::Selector = scraper::Selector::parse("a:not([href^=\"#\"])").unwrap();

#[derive(Debug)]
struct CrawlerError(reqwest::Error);

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Custom Error: {}", self.0)
    }
}

impl std::error::Error for CrawlerError {}

#[tokio::main]
async fn main() {
    // let selector = scraper::Selector::parse("a:not([href^=\"#\"])").unwrap();
    // let webpage = reqwest::get("https://github.com").await?.text().await?;
    // let doc = scraper::Html::parse_document(&webpage);

    // for el in doc.select(&selector) {
    //     let text = el.text().collect::<Vec<_>>();
    //     let link = el.value().attr("href").unwrap();
    //     println!("{} ({})", text[0], link);
    // }
    // println!("{}", doc.select(&selector).count());

    let mut db = Db {
        nodes: Nodes::new(),
        links: Links::new(),
    };

    match crawl("https://github.com", &mut db, 0, 1).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

#[async_recursion::async_recursion(?Send)]
async fn crawl(url: &str, db: &mut Db, depth: i32, group_num: i32) -> Result<(), CrawlerError> {
    let webpage = match reqwest::get(url).await {
        Ok(w) => w,
        Err(e) => {
            return Err(CrawlerError(e));
        }
    };
    let text = match webpage.text().await {
        Ok(t) => t,
        Err(e) => {
            return Err(CrawlerError(e));
        }
    };

    let doc = scraper::Html::parse_document(&text);

    for el in doc.select(&SELECTOR) {
        let next_url = el.value().attr("href").unwrap();
        let node = Node {
            id: next_url.to_string(),
            group: group_num,
        };
        let link = Link {
            source: url.to_string(),
            target: next_url.to_string(),
            value: 1,
        };

        if db.links.contains(&link) {
            continue;
        }

        if db.nodes.contains(&node) {
            db.links.insert(link);
            continue;
        } else {
            db.nodes.insert(node);
        }

        if depth >= MAX_DEPTH {
            continue;
        }

        let mut _grp_num: i32;
        if !next_url.starts_with("/") {
            _grp_num = group_num + 1;
        } else {
            _grp_num = group_num;
        }

        match crawl(next_url, db, depth + 1, _grp_num).await {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}
