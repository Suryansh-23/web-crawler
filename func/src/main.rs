use std::collections::{HashMap, HashSet};

use crate::crawl::{Db, Links, Node, Nodes, SELECTOR_PATTERN};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

mod crawl;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let algo = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("type"))
        .unwrap_or("bfs");

    let url = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("url"));
    if url.is_none() {
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "text/html")
            .body("url is required".into())
            .map_err(Box::new)?);
    }

    let message = format!("Hello {algo}, this is an AWS Lambda HTTP request");
    let selector = scraper::Selector::parse(SELECTOR_PATTERN).unwrap();

    let mut db = Db {
        nodes: Nodes::new(),
        links: Links::new(),
        host_names: HashSet::new(),
        freq_table: HashMap::new(),
    };
    db.nodes.insert(Node {
        url: url.unwrap_or("").to_string(),
        group: 0,
    });

    if algo == "bfs" {
        let resp = Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(message.into())
            .map_err(Box::new)?;
        Ok(resp)
    } else {
        let resp = Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(message.into())
            .map_err(Box::new)?;
        Ok(resp)
    }

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
