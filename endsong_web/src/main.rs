use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use endsong::prelude::*;
use rinja_axum::Template;
use tokio::{fs::File, io::AsyncReadExt, sync::RwLock};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Clone)]
struct AppState {
    entries: Arc<RwLock<SongEntries>>,
}

#[tokio::main]
async fn main() {
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env).init();

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

    let state = Arc::new(AppState {
        entries: Arc::new(RwLock::new(entries)),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .with_state(state)
        .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

/// [`Template`] for [`not_found`]
#[derive(Template)]
#[template(path = "404.html", print = "none")]
struct NotFound;
/// 404
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, NotFound {})
}

/// GET `/styles` - CSS
///
/// Idk yet how, but should be cached somehow for the future so that
/// it isn't requested on each load in full? idk
async fn styles() -> impl IntoResponse {
    tracing::debug!("GET /styles");
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/css").unwrap());

    let mut file = File::open("templates/tailwind_style.css").await.unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();

    (headers, contents)
}

/// [`Template`] for [`index`]
#[derive(Template)]
#[template(path = "index.html", print = "none")]
struct Index {
    total_listened: TimeDelta,
    playcount: usize,
}
/// GET `/`
async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    tracing::debug!("GET /");
    let entries = state.entries.read().await;
    Index {
        total_listened: gather::listening_time(&entries),
        playcount: gather::all_plays(&entries),
    }
}
