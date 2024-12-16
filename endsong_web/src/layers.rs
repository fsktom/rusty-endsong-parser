//! Contains custom [`tower`] [`tower::Layer`]s used here
//! or functions used later with [`axum::middleware::from_fn`]

use axum::{
    body::{self, Body},
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::Next,
    response::Response,
};
use regex::Regex;
use tracing::trace;

/// Removes HTML comments from `text/html` responses
///
/// # Panics
///
/// I hope it doesn't panic.
pub async fn remove_html_comments(request: Request, next: Next) -> Response {
    trace!("removing HTML comments");
    let response = next.run(request).await;
    let headers = response.headers().clone();

    let Some(content_type) = response.headers().get(CONTENT_TYPE) else {
        return response;
    };

    // thanks chat gippity
    if content_type.to_str().unwrap_or("").starts_with("text/html") {
        if let Ok(body_bytes) = body::to_bytes(response.into_body(), usize::MAX).await {
            let body_str = String::from_utf8_lossy(&body_bytes);
            // Use regex to remove HTML comments
            let re = Regex::new(r"<!--.*?-->").unwrap();
            let cleaned_body = re.replace_all(&body_str, "").to_string();

            // Rebuild the response with the cleaned body
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
