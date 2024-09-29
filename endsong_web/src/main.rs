use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use endsong::prelude::*;
use rinja_axum::Template;
use tokio::sync::RwLock;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[tokio::main]
async fn main() {
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env).init();

    // let data = Data {
    //     entries: Arc::new(RwLock::new(entries)),
    // };

    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    // different root path depending on my OS
    let root = match std::env::consts::OS {
        "windows" => r"C:\Temp\Endsong\",
        "macos" => "/Users/filip/Other/Endsong/",
        _ => "/mnt/c/temp/Endsong/",
    };
    let last: u8 = 0;
    let paths: Vec<String> = (0..=last)
        .map(|i| format!("{root}endsong_{i}.json"))
        .collect();

    let entries = SongEntries::new(&paths)
        .unwrap_or_else(|e| panic!("{e}"))
        .sum_different_capitalization()
        .filter(30, TimeDelta::try_seconds(10).unwrap());

    Index {
        total_listened: gather::listening_time(&entries),
        playcount: gather::all_plays(&entries),
    }
}

#[derive(Template)]
#[template(path = "index.html", print = "none")]
struct Index {
    total_listened: TimeDelta,
    playcount: usize,
}

// no worky bc using Rc<str> in `endsong`
// #[derive(Clone)]
// struct Data {
//     entries: Arc<RwLock<SongEntries>>,
// }
