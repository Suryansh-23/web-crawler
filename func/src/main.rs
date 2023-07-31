#![deny(clippy::all)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::{collections::HashSet, fmt};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    url: String,
    group: i32,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

type Nodes = HashSet<Node>;

#[derive(Debug, Serialize, Deserialize)]
struct Link {
    source: String,
    target: String,
}

impl Eq for Link {}

impl PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.target == other.target
    }
}

impl Hash for Link {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.target.hash(state);
    }
}

type Links = HashSet<Link>;

#[derive(Debug, Serialize, Deserialize)]
struct Db {
    nodes: Nodes,
    links: Links,
    host_names: HashSet<String>,
}

const MAX_DEPTH: i32 = 10;
const MAX_NODES: i32 = 1000;

#[derive(Debug)]
struct CrawlerError(reqwest::Error);

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Custom Error: {}", self.0)
    }
}

impl std::error::Error for CrawlerError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://rust-lang.org/";
    let selector = scraper::Selector::parse("a:not([href^=\"#\"])").unwrap();

    let mut db = Db {
        nodes: Nodes::new(),
        links: Links::new(),
        host_names: HashSet::new(),
    };
    db.nodes.insert(Node {
        url: url.to_string(),
        group: 0,
    });

    // match crawl_dfs(Url::parse(url).unwrap(), &selector, &mut db, 0, 1).await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
    match crawl_bfs(Url::parse(url).unwrap(), &selector, &mut db).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("nodes count: {}", db.nodes.len());
    println!("links count: {}", db.links.len());

    let mut fp = std::fs::File::create("V:\\Projects\\Web-Crawler\\13f816d9685cdef3\\data.json")?;
    write!(fp, "{}", serde_json::to_string(&db)?);

    Ok(())
}

#[async_recursion::async_recursion(?Send)]
async fn crawl_dfs(
    url: Url,
    selector: &scraper::Selector,
    db: &mut Db,
    depth: i32,
    group_num: i32,
) -> Result<(), CrawlerError> {
    let webpage = match reqwest::get(url.as_str()).await {
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

    for el in doc.select(selector) {
        if db.nodes.len() >= MAX_NODES as usize {
            return Ok(());
        }

        let raw_url = match el.value().attr("href") {
            Some(u) => u,
            None => {
                println!("No href found {:?}\n", el.value());
                continue;
            }
        };
        // println!("{}", raw_url);
        let next_url = url.join(raw_url).unwrap();
        let node = Node {
            url: next_url.to_string(),
            group: group_num,
        };
        let link = Link {
            source: url.to_string(),
            target: next_url.to_string(),
        };

        if db.links.contains(&link) {
            continue;
        }

        db.links.insert(link);
        if db.nodes.contains(&node) {
            continue;
        } else {
            db.nodes.insert(node);
            db.host_names
                .insert(next_url.host_str().unwrap().to_string());
        }

        if depth >= MAX_DEPTH {
            continue;
        }

        if raw_url.starts_with("/") {
            match crawl_dfs(next_url, selector, db, depth + 1, group_num).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            match crawl_dfs(next_url, selector, db, depth + 1, group_num + 1).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

#[async_recursion::async_recursion(?Send)]
async fn crawl_bfs(
    start_url: Url,
    selector: &scraper::Selector,
    db: &mut Db,
) -> Result<(), CrawlerError> {
    let mut queue = VecDeque::new();
    queue.push_back((start_url, 0, 0)); // (url, depth, group_num)

    while let Some((url, depth, group_num)) = queue.pop_front() {
        if db.nodes.len() >= MAX_NODES as usize {
            return Ok(());
        }

        let webpage = reqwest::get(url.as_str()).await.unwrap();
        let text = webpage.text().await.unwrap_or(String::from(""));

        let doc = scraper::Html::parse_document(&text);

        for el in doc.select(selector) {
            if db.nodes.len() >= MAX_NODES as usize {
                return Ok(());
            }
            let raw_url = match el.value().attr("href") {
                Some(u) => u,
                None => {
                    println!("No href found {:?}\n", el.value());
                    continue;
                }
            };
            let next_url = url.join(raw_url).unwrap();
            let node = Node {
                url: next_url.to_string(),
                group: group_num,
            };
            let link = Link {
                source: url.to_string(),
                target: next_url.to_string(),
            };

            if db.links.contains(&link) {
                continue;
            }

            db.links.insert(link);
            if db.nodes.contains(&node) {
                continue;
            } else {
                db.nodes.insert(node);
                db.host_names
                    .insert(next_url.host_str().unwrap_or("").to_string());
            }

            if depth >= MAX_DEPTH {
                continue;
            }

            if raw_url.starts_with("/") {
                queue.push_back((next_url, depth + 1, group_num));
            } else {
                queue.push_back((next_url, depth + 1, group_num + 1));
            }
        }
    }

    Ok(())
}
