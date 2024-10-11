//! Contains templates for `/artists`` routes

use crate::AppState;

use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
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

#[derive(Deserialize)]
pub struct ArtistListForm {
    search: String,
}
/// [`Template`] for [`elements`]
///
/// Template:
/// ```rinja
/// {% for artist in artist_names %}
/// <li><a href="/artist/{{ artist|encodeurl }}">{{ artist }}</a></li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct ElementsTemplate {
    artist_names: Vec<Arc<str>>,
}
/// POST `/artists`
///
/// List of artists
pub async fn elements(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ArtistListForm>,
) -> impl IntoResponse {
    debug!(search = form.search, "POST /artists");

    let artists = state.artists.read().await;

    let lowercase_search = form.search.to_lowercase();

    let artist_names = artists
        .iter()
        .filter(|artist| artist.to_lowercase().contains(&lowercase_search))
        .cloned()
        .collect();

    ElementsTemplate { artist_names }
}

mod filters {
    use urlencoding::encode;

    pub fn encodeurl(name: &str) -> rinja::Result<String> {
        // bc of artists like AC/DC
        Ok(encode(name).to_string())
    }
}
