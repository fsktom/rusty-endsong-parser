pub mod artist;
pub mod artists;
pub mod r#static;

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use endsong::prelude::*;
use rinja::Template;
use tokio::sync::RwLock;
use tracing::debug;

/// State shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Reference to the [`SongEntries`] instance used
    pub entries: Arc<RwLock<SongEntries>>,
    /// Sorted list of all artist names in the dataset
    pub artists: Arc<RwLock<Vec<Arc<str>>>>,
}
impl AppState {
    pub fn new(entries: SongEntries) -> Arc<Self> {
        Arc::new(Self {
            artists: Arc::new(RwLock::new(entries.artists())),
            entries: Arc::new(RwLock::new(entries)),
        })
    }
}

/// [`Template`] for [`not_found`]
#[derive(Template)]
#[template(path = "404.html", print = "none")]
struct NotFoundTemplate;
/// 404
pub async fn not_found() -> impl IntoResponse {
    debug!("404");

    (StatusCode::NOT_FOUND, NotFoundTemplate {})
}

/// [`Template`] for [`index`]
#[derive(Template)]
#[template(path = "index.html", print = "none")]
struct IndexTemplate {
    total_listened: TimeDelta,
    playcount: usize,
}
/// GET `/`
pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("GET /");

    let entries = state.entries.read().await;

    IndexTemplate {
        total_listened: gather::total_listening_time(&entries),
        playcount: gather::all_plays(&entries),
    }
}
