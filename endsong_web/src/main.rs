use endsong_web::*;

use axum::{routing::get, routing::post, Router};
use endsong::prelude::*;
use tower_http::compression::CompressionLayer;
use tracing::debug;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

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

    let state = AppState::new(entries);

    let compression = CompressionLayer::new().br(true);

    let app = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(r#static::styles))
        .route("/htmx.js", get(r#static::htmx))
        .route("/artists", get(artists::base))
        .route("/artists", post(artists::elements))
        .route("/artist/:artist_name", get(artist::base))
        .with_state(state)
        .fallback(not_found)
        .layer(compression);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
