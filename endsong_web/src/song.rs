//! Song page
//!
//! Contains handlers for `/song` routes

#![allow(clippy::module_name_repetitions, reason = "looks nicer")]

use crate::AppState;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use endsong::prelude::*;
use rinja_axum::Template;

/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "song.html")]
struct SongTemplate {
    /// List of songs
    song_versions: Vec<(Song, usize)>,
}
/// GET `/song/[:artist_name]/[:song_name][?artist_id=usize][&song_id=usize]`
///
/// # Panics
///
/// Shouldn't panic lol
pub async fn base(
    State(state): State<Arc<AppState>>,
    Path((artist_name, song_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let entries = &state.entries;

    let songs = entries.find().song(&song_name, &artist_name).unwrap();

    let song_versions = songs
        .iter()
        .map(|song| (song.clone(), gather::plays(entries, song)))
        .collect();

    SongTemplate { song_versions }
}
