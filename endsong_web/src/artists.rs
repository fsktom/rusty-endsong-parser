//! Contains templates for `/artists` routes

use crate::AppState;

use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use endsong::prelude::*;
use rinja::Template;
use serde::Deserialize;
use tracing::debug;

/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "artists.html", print = "none")]
struct BaseTemplate {}
/// GET `/artists`
///
/// List of artists (HTML Template will call [`elements`] on-load)
pub async fn base() -> impl IntoResponse {
    debug!("GET /artists");

    BaseTemplate {}
}

/// [`Form`] arguments sent by an `input` used in [`elements`]
#[derive(Deserialize)]
pub struct ArtistListForm {
    /// Search query
    search: String,
}
/// [`Template`] for [`elements`]
///
/// Template:
/// ```rinja
/// {% for (link, artist, plays) in artists %}
/// <li><a href="{{ link }}">{{ artist.name }} | {{ plays }} plays</a></li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct ElementsTemplate {
    /// List of artists constrained by the query
    ///
    /// Elements: Link to artist page, [`Artist`] instance, playcount
    artists: Vec<(String, Artist, usize)>,
}
/// POST `/artists`
///
/// List of artists
#[expect(
    clippy::missing_panics_doc,
    reason = "will not panic since guaranteed artist in HashMap"
)]
pub async fn elements(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ArtistListForm>,
) -> impl IntoResponse {
    debug!(search = form.search, "POST /artists");

    let lowercase_search = form.search.to_lowercase();

    let artists = state
        .artists
        .iter()
        .filter(|artist| artist.name.to_lowercase().contains(&lowercase_search))
        .map(|artist| {
            (
                format!("/artist/{artist}"),
                artist.clone(),
                state.artist_ranking.get(artist).unwrap().0,
            )
        })
        .collect();

    ElementsTemplate { artists }
}
