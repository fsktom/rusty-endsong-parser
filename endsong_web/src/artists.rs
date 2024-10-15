//! Contains templates for `/artists` routes

use crate::{AppState, ArtistInfo};

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
/// {% for (artist, info) in artists %}
/// <li>
/// <a href="{{ info.link }}"
///   >{{ artist }} | {{ info.plays }} play{{ info.plays|pluralize }} | {{
///   info.duration.num_minutes() }} minute{{
///   info.duration.num_minutes()|pluralize }}</a
/// >
/// </li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct ElementsTemplate {
    /// List of artists constrained by the query
    ///
    /// Elements: Link to artist page, [`Artist`] instance, playcount
    artists: Vec<(Artist, ArtistInfo)>,
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
                artist.clone(),
                state.artist_info.get(artist).unwrap().clone(),
            )
        })
        .collect();

    ElementsTemplate { artists }
}
