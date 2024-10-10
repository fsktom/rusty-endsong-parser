use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use endsong::prelude::*;
use rinja_axum::Template;
use tokio::sync::RwLock;
use tower_http::compression::CompressionLayer;
use tracing::debug;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

/// Tailwind-generated CSS used on this web page
const STYLING: &str = include_str!("../templates/tailwind_style.css");

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

    axum_extra::response::Css(STYLING)
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
        total_listened: gather::total_listening_time(&entries),
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

    let artist_names = entries.artists();

    Artists { artist_names }
}

/// To choose an artist if there are multiple with same capitalization
/// (in my dataset tia)
#[derive(serde::Deserialize)]
struct ArtistQuery {
    id: usize,
}
/// [`Template`] for if there are multiple artist with different
/// capitalization in [`artist`]
#[derive(Template)]
#[template(path = "artist_selection.html", print = "none")]
struct ArtistSelection {
    artists: Vec<Artist>,
}
/// [`Template`] for [`artist`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistPage<'a> {
    artist: &'a Artist,
    plays: usize,
    time_played: TimeDelta,
}
/// GET `/artist/:artist_name(?id=usize)`
///
/// Artist page
///
/// Returns an [`ArtistPage`] with a valid `artist_name`,
/// an [`ArtistSelection`] if there are multiple artists with this name
/// but different capitalization,
/// and [`not_found`] if it's not in the dataset
async fn artist(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    options: Option<Query<ArtistQuery>>,
) -> Response {
    debug!(
        artist_name = artist_name,
        query = options.is_some(),
        "GET /artist/:artist_name(?query)"
    );

    let entries = state.entries.read().await;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(Query(options)) = options {
        artists.get(options.id)
    } else {
        None
    };

    let artist = if let Some(artist) = artist {
        artist
    } else {
        // query if multiple artists with different capitalization
        return ArtistSelection { artists }.into_response();
    };

    ArtistPage {
        plays: gather::plays(&entries, artist),
        time_played: gather::listening_time(&entries, artist),
        artist,
    }
    .into_response()
}

mod filters {
    use urlencoding::encode;

    pub fn encodeurl(name: &str) -> rinja::Result<String> {
        // bc of artists like AC/DC
        Ok(encode(name).to_string())
    }
}
