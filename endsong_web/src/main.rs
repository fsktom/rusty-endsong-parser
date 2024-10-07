use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use endsong::prelude::*;
use itertools::Itertools;
use rinja_axum::Template;
use tokio::{fs::File, io::AsyncReadExt, sync::RwLock};
use tower_http::compression::CompressionLayer;
use tracing::debug;
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

    let compression = CompressionLayer::new().br(true);

    let app = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/artists", get(artists))
        .route("/artist/:artist_name", get(artist))
        .with_state(state)
        .fallback(not_found)
        .layer(compression);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

/// [`Template`] for [`not_found`]
#[derive(Template)]
#[template(path = "404.html", print = "none")]
struct NotFound;
/// 404
async fn not_found() -> impl IntoResponse {
    debug!("404");
    (StatusCode::NOT_FOUND, NotFound {})
}

/// GET `/styles` - CSS
///
/// Idk yet how, but should be cached somehow for the future so that
/// it isn't requested on each load in full? idk
async fn styles() -> impl IntoResponse {
    debug!("GET /styles");
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
    debug!("GET /");
    let entries = state.entries.read().await;
    Index {
        total_listened: gather::listening_time(&entries),
        playcount: gather::all_plays(&entries),
    }
}

/// [`Template`] for [`artists`]
#[derive(Template)]
#[template(path = "artists.html", print = "none")]
struct Artists {
    artist_names: Vec<Arc<str>>,
}
/// GET `/artists`
///
/// List of artists
async fn artists(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("GET /artists");
    let entries = state.entries.read().await;

    let artist_names = entries.artists().into_iter().sorted_unstable().collect();

    Artists { artist_names }
}

/// [`Template`] for [`artist`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistPage {
    artist: Artist,
    plays: usize,
    time_played: TimeDelta,
}
/// GET `/artist/:artist_name`
///
/// Artist page
///
/// Returns an [`ArtistPage`] with a valid `artist_name`
/// and [`not_found`] if it's not in the dataset
async fn artist(State(state): State<Arc<AppState>>, Path(artist_name): Path<String>) -> Response {
    debug!("GET /artist/{}", artist_name);
    let entries = state.entries.read().await;

    match entries.find().artist(&artist_name) {
        Some(artist) => {
            let artist = artist[0].clone();
            ArtistPage {
                plays: gather::plays(&entries, &artist),
                time_played: entries
                    .iter()
                    .filter(|e| artist.is_entry(e))
                    .map(|e| e.time_played)
                    .sum(),
                artist,
            }
            .into_response()
        }
        None => not_found().await.into_response(),
    }
}
