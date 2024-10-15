//! ENDSONG CAN INTO WEB

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
#![allow(clippy::unused_async, reason = "axum handlers must be async")]

pub mod album;
pub mod artist;
pub mod artists;
pub mod r#static;

use std::collections::HashMap;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use endsong::prelude::*;
use itertools::Itertools;
use rinja::Template;
use serde::{Deserialize, Deserializer};
use tracing::debug;

/// State shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Reference to the [`SongEntries`] instance used
    pub entries: Arc<SongEntries>,
    /// Sorted (ascending alphabetically) list of all artists in the dataset
    pub artists: Arc<Vec<Artist>>,
    /// Map of artists with their playcount (.0) and their position/ranking descending (.1)
    pub artist_ranking: Arc<HashMap<Artist, (usize, usize)>>,
}
impl AppState {
    /// Creates a new [`AppState`] within an [`Arc`]
    #[must_use]
    pub fn new(entries: SongEntries) -> Arc<Self> {
        let artist_ranking: HashMap<Artist, (usize, usize)> = gather::artists(&entries)
            .into_iter()
            .sorted_unstable_by_key(|(art, plays)| (std::cmp::Reverse(*plays), art.clone()))
            .enumerate()
            // bc enumeration starts with 0 :P
            .map(|(position, (art, plays))| (art, (plays, position + 1)))
            .collect();

        Arc::new(Self {
            artist_ranking: Arc::new(artist_ranking),
            artists: Arc::new(
                entries
                    .iter()
                    .map(Artist::from)
                    .unique()
                    .sorted_unstable()
                    .collect_vec(),
            ),
            entries: Arc::new(entries),
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
    /// Total time listened
    total_listened: TimeDelta,
    /// Total playcount
    playcount: usize,
}
/// GET `/`
pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("GET /");

    let entries = &state.entries;

    IndexTemplate {
        total_listened: gather::total_listening_time(entries),
        playcount: gather::all_plays(entries),
    }
}

/// Whether to sort by playcount or time listened
#[derive(Debug)]
enum Sorting {
    /// Sort by playcount
    Plays,
    /// Sort by time listened
    Minutes,
}
impl std::fmt::Display for Sorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plays => write!(f, "plays"),
            Self::Minutes => write!(f, "minutes"),
        }
    }
}
impl<'de> Deserialize<'de> for Sorting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "minutes" => Ok(Self::Minutes),
            _ => Ok(Self::Plays),
        }
    }
}
/// Form used in [`top_artists`]
#[derive(Deserialize)]
pub struct TopArtistsForm {
    /// Number of artists to display
    top: Option<usize>,
    /// Way to sort the artits
    sort: Sorting,
}
/// [`Template`] for [`top_artists`]
///
/// ```rinja
/// {% for (link, artist, plays, minutes) in artists %}
/// <li class="ml-7"><a href="{{ link }}">{{ artist }} | {{ plays }} play{{ plays|pluralize }} | {{ minutes }} minute{{ minutes|pluralize }}</a></li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct TopArtistsTempate {
    /// Elements: link to artist page, [`Artist`] instance, play count, number of minutes
    artists: Vec<(String, Artist, usize, i64)>,
}
/// POST `/top_artists[?top=usize][&sort=String]`
pub async fn top_artists(
    State(state): State<Arc<AppState>>,
    Form(form): Form<TopArtistsForm>,
) -> impl IntoResponse {
    debug!(
        top = form.top,
        sort = form.sort.to_string(),
        "POST /top_artists[?top=usize][&sort=Sorting]"
    );

    let entries = &state.entries;

    let top = form.top.unwrap_or(10000);

    let artists = match form.sort {
        Sorting::Plays => gather::artists(entries)
            .into_iter()
            .sorted_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
            .take(top)
            .map(|(artist, plays)| {
                (
                    format!("/artist/{artist}"),
                    artist.clone(),
                    plays,
                    gather::listening_time(entries, &artist).num_minutes(),
                )
            })
            .collect(),
        Sorting::Minutes => gather::artists(entries)
            .into_iter()
            .map(|(artist, plays)| {
                (
                    format!("/artist/{artist}"),
                    artist.clone(),
                    plays,
                    gather::listening_time(entries, &artist).num_minutes(),
                )
            })
            .sorted_unstable_by(|a, b| b.3.cmp(&a.3).then_with(|| a.1.cmp(&b.1)))
            .take(top)
            .collect(),
    };

    TopArtistsTempate { artists }
}

/// Custom URL encoding
///
/// Mostly for encoding `/` in something like `AC/DC`
/// to make a working link
#[must_use]
pub fn encode_url(name: &str) -> String {
    urlencoding::encode(name).into_owned()
}
