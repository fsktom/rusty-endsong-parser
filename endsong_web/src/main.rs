//! WEB

// unsafe code is bad
#![deny(unsafe_code)]
// can be a pain, but it's worth it
// don't forget to use #[expect(clippy::...)] when sensible
#![warn(clippy::pedantic)]
// because I want to be explicit when cloning is cheap
#![warn(clippy::clone_on_ref_ptr)]
// doc lints, checked when compiling/running clippy
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
// other doc lints, only checked when building docs
// https://doc.rust-lang.org/rustdoc/lints.html
// other good ones are warn by default
#![warn(rustdoc::missing_crate_level_docs, rustdoc::unescaped_backticks)]
// https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html#expectlint
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::allow_attributes)]

use endsong_web::{album, artist, artists, history, r#static, song};
use endsong_web::{index, not_found, top_artists, AppState};

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
    let last: u8 = 9;
    let paths: Vec<String> = (0..=last)
        .map(|i| format!("{root}endsong_{i}.json"))
        .collect();

    let entries = SongEntries::new(&paths)
        .unwrap_or_else(|e| panic!("{e}"))
        // .sum_different_capitalization()
        .filter(30, TimeDelta::seconds(10));

    let state = AppState::new(entries);

    let compression = CompressionLayer::new().br(true);

    let app = Router::new()
        .route("/", get(index))
        .route("/top_artists", post(top_artists))
        .route("/styles.css", get(r#static::styles))
        .route("/htmx.js", get(r#static::htmx))
        .route("/plotly.js", get(r#static::plotly))
        .route("/artists", get(artists::base).post(artists::elements))
        .route("/artist/:artist_name", get(artist::base))
        .route("/artist/:artist_name/albums", post(artist::albums))
        .route("/artist/:artist_name/songs", post(artist::songs))
        .route(
            "/artist/:artist_name/absolute_plot",
            post(artist::absolute_plot),
        )
        .route(
            "/artist/:artist_name/relative_plot",
            post(artist::relative_plot),
        )
        .route("/album/:artist_name/:album_name", get(album::base))
        .route("/song/:artist_name/:song_name", get(song::base))
        .route("/history", get(history::base).post(history::elements))
        .route("/history/datepicker", post(history::date_picker))
        .with_state(state)
        .fallback(not_found)
        .layer(compression);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
