//! Contains routes for static files

use axum::response::IntoResponse;
use tracing::debug;

/// Tailwind-generated CSS used on this web page
const STYLING: &str = include_str!("../static/tailwind_style.css");

/// HTMX code (<https://htmx.org/docs/#installing>)
const HTMX: &str = include_str!("../static/htmx.min.2.0.3.js");

/// GET `/styles.css` - CSS
///
/// Idk yet how, but should be cached somehow for the future so that
/// it isn't requested on each load in full? idk
pub async fn styles() -> impl IntoResponse {
    debug!("GET /styles.css");

    axum_extra::response::Css(STYLING)
}

/// GET `/htmx.js` - HTMX
///
/// Idk yet how, but should be cached somehow for the future so that
/// it isn't requested on each load in full? idk
pub async fn htmx() -> impl IntoResponse {
    debug!("GET /htmx.js");

    axum_extra::response::JavaScript(HTMX)
}
