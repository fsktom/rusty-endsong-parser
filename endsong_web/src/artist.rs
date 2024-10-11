//! Contains templates for `/artist` route

use crate::*;

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use endsong::prelude::*;
use rinja_axum::Template;
use serde::Deserialize;
use tracing::debug;

/// To choose an artist if there are multiple with same capitalization
/// (in my dataset tia)
#[derive(Deserialize)]
pub struct ArtistQuery {
    id: usize,
}
/// [`Template`] for if there are multiple artist with different
/// capitalization in [`artist`]
#[derive(Template)]
#[template(path = "artist_selection.html", print = "none")]
struct ArtistSelectionTemplate {
    artists: Vec<Artist>,
}
/// [`Template`] for [`artist`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistTemplate<'a> {
    artist: &'a Artist,
    plays: usize,
    time_played: TimeDelta,
}
/// GET `/artist/:artist_name(?id=usize)`
///
/// Artist page
///
/// Returns an [`ArtistTemplate`] with a valid `artist_name`,
/// an [`ArtistSelectionTemplate`] if there are
/// multiple artists with this name
/// but different capitalization,
/// and [`not_found`] if it's not in the dataset
pub async fn base(
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
        return ArtistSelectionTemplate { artists }.into_response();
    };

    ArtistTemplate {
        plays: gather::plays(&entries, artist),
        time_played: gather::listening_time(&entries, artist),
        artist,
    }
    .into_response()
}
