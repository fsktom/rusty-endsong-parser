//! Contains custom [`tower`] [`tower::Layer`]s used here
//! or functions used later with [`axum::middleware::from_fn`]

use axum::{
    body::{self, Body},
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::Next,
    response::Response,
};
use minify_html::{minify, Cfg};
use tracing::trace;

/// Minifies HTML in `text/html` responses
///
/// # Panics
///
/// I hope it doesn't panic.
pub async fn minify_html(request: Request, next: Next) -> Response {
    trace!("minifying HTML");
    let response = next.run(request).await;
    let headers = response.headers().clone();

    let Some(content_type) = response.headers().get(CONTENT_TYPE) else {
        return response;
    };

    // thanks chat gippity
    if content_type.to_str().unwrap_or("").starts_with("text/html") {
        if let Ok(body_bytes) = body::to_bytes(response.into_body(), usize::MAX).await {
            let mut cfg = Cfg::new();
            cfg.minify_css = true;
            cfg.minify_js = false;
            let minified = minify(&body_bytes, &cfg);
            let cleaned_body = String::from_utf8_lossy(&minified).to_string();

            let body = Body::from(cleaned_body);
            let mut response = Response::builder();
            let h = response.headers_mut().unwrap();
            *h = headers;
            response.body(body).unwrap()
        } else {
            Response::new("body".into())
        }
    } else {
        response
    }
}

/// Minifies CSS in `text/css` responses
///
/// # Panics
///
/// I hope it doesn't panic.
pub async fn minify_css(request: Request, next: Next) -> Response {
    trace!("minifying CSS");
    let response = next.run(request).await;
    let headers = response.headers().clone();

    let Some(content_type) = response.headers().get(CONTENT_TYPE) else {
        return response;
    };

    // thanks chat gippity
    if content_type.to_str().unwrap_or("").starts_with("text/css") {
        if let Ok(body_bytes) = body::to_bytes(response.into_body(), usize::MAX).await {
            let mut cfg = Cfg::new();
            cfg.minify_css = true;
            cfg.minify_js = false;
            let minified = minify(&body_bytes, &cfg);
            let cleaned_body = String::from_utf8_lossy(&minified).to_string();

            let body = Body::from(cleaned_body);
            let mut response = Response::builder();
            let h = response.headers_mut().unwrap();
            *h = headers;
            response.body(body).unwrap()
        } else {
            Response::new("body".into())
        }
    } else {
        response
    }
}
