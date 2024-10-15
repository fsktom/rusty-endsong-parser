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
    /// Map of artists with the link to thoer page, their playcount their position/ranking descending
    pub artist_info: Arc<HashMap<Artist, ArtistInfo>>,
}
impl AppState {
    /// Creates a new [`AppState`] within an [`Arc`]
    #[must_use]
    pub fn new(entries: SongEntries) -> Arc<Self> {
        let mut artist_info: HashMap<Artist, ArtistInfo> =
            HashMap::with_capacity(entries.len() / 100);

        let artists_with_duration = gather::artists_with_duration(&entries);
        for (position, (artist, (plays, duration))) in artists_with_duration
            .iter()
            .sorted_unstable_by_key(|(art, (plays, _))| (std::cmp::Reverse(*plays), *art))
            .enumerate()
        {
            artist_info.entry(artist.clone()).or_insert(ArtistInfo {
                link: Arc::from(format!("/artist/{artist}")),
                plays: *plays,
                duration: *duration,
                // bc enumerate starts with 0
                position_plays: position + 1,
                position_duration: 0,
            });
        }
        for (position, (artist, _)) in artists_with_duration
            .into_iter()
            .sorted_unstable_by_key(|(art, (_, duration))| {
                (std::cmp::Reverse(*duration), art.clone())
            })
            .enumerate()
        {
            artist_info
                .entry(artist)
                .and_modify(|e| e.position_duration = position + 1);
        }

        Arc::new(Self {
            artist_info: Arc::new(artist_info),
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

/// Used in [`AppState`] for artist in a [`HashMap`]
#[derive(Clone)]
pub struct ArtistInfo {
    /// Link to the artist page (i.e. `/artist/[:artist_name]`)
    link: Arc<str>,
    /// This artist's playcount
    plays: usize,
    /// Position in regards to the playcount
    position_plays: usize,
    /// Total time listened to this aritst
    duration: TimeDelta,
    /// Position in regards to the time listened
    position_duration: usize,
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
/// {% for (artist, info) in artists %}
/// <li class="ml-7">
///   <a href="{{ info.link }}"
///     >{{ artist }} | {{ info.plays }} play{{ info.plays|pluralize }} | {{
///     info.duration.num_minutes() }} minute{{
///     info.duration.num_minutes()|pluralize }}</a
///   >
/// </li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct TopArtistsTempate {
    /// List of [`Artist`]s with their info
    artists: Vec<(Artist, ArtistInfo)>,
}
/// POST `/top_artists[?top=usize][&sort=String]`
#[expect(
    clippy::missing_panics_doc,
    reason = "unwraps which should never panic"
)]
pub async fn top_artists(
    State(state): State<Arc<AppState>>,
    Form(form): Form<TopArtistsForm>,
) -> impl IntoResponse {
    debug!(
        top = form.top,
        sort = form.sort.to_string(),
        "POST /top_artists[?top=usize][&sort=Sorting]"
    );

    let top = form.top.unwrap_or(10000);

    let artists = match form.sort {
        Sorting::Plays => state
            .artists
            .iter()
            .map(|artist| {
                (
                    artist.clone(),
                    state.artist_info.get(artist).unwrap().clone(),
                )
            })
            .sorted_unstable_by(|a, b| b.1.plays.cmp(&a.1.plays).then_with(|| a.0.cmp(&b.0)))
            .take(top)
            .collect(),
        Sorting::Minutes => state
            .artists
            .iter()
            .map(|artist| {
                (
                    artist.clone(),
                    state.artist_info.get(artist).unwrap().clone(),
                )
            })
            .sorted_unstable_by(|a, b| b.1.duration.cmp(&a.1.duration).then_with(|| a.0.cmp(&b.0)))
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
