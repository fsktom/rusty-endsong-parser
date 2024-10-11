//! Contains routes for static files

use axum::{
    http::header::CACHE_CONTROL,
    response::{AppendHeaders, IntoResponse},
};
use tracing::debug;

/// Tailwind-generated CSS used on this web page
///
/// `npx tailwindcss -i base_style.css -o ../static/tailwind_style.css --watch`
const STYLING: &str = include_str!("../static/tailwind_style.css");

/// HTMX code
///
/// <https://htmx.org/docs/#installing>
const HTMX: &str = include_str!("../static/htmx.min.2.0.3.js");

/// plotly.js code
///
/// <https://github.com/plotly/plotly.js#load-via-script-tag>
const PLOTLY: &str = include_str!("../static/plotly-2.35.2.min.js");

/// GET `/styles.css` - CSS
///
/// Idk yet how, but should be cached somehow for the future so that
/// it isn't requested on each load in full? idk (but also is invalidated in rapid dev..)
pub async fn styles() -> impl IntoResponse {
    debug!("GET /styles.css");

    axum_extra::response::Css(STYLING)
}

/// GET `/htmx.js` - HTMX
pub async fn htmx() -> impl IntoResponse {
    debug!("GET /htmx.js");

    let headers = AppendHeaders([(CACHE_CONTROL, "public, max-age=31536000, immutable")]);

    (headers, axum_extra::response::JavaScript(HTMX))
}

/// GET `/plotly.js` - plotly.js
pub async fn plotly() -> impl IntoResponse {
    debug!("GET /plotly.js");

    let headers = AppendHeaders([(CACHE_CONTROL, "public, max-age=31536000, immutable")]);

    (headers, axum_extra::response::JavaScript(PLOTLY))
}
