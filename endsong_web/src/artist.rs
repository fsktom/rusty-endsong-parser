//! Contains templates for `/artist` route

#![allow(clippy::module_name_repetitions, reason = "looks nicer")]

use crate::{not_found, AppState};

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use endsong::prelude::*;
use plotly::{Layout, Plot, Scatter};
use rinja_axum::Template;
use serde::Deserialize;
use tracing::debug;

/// To choose an artist if there are multiple with same capitalization
/// (in my dataset tia)
#[derive(Deserialize)]
pub struct ArtistQuery {
    /// The artist's index in the [`Vec`] returned by [`find::artist`]
    id: usize,
}
/// [`Template`] for if there are multiple artist with different
/// capitalization in [`base`]
#[derive(Template)]
#[template(path = "artist_selection.html", print = "none")]
struct ArtistSelectionTemplate {
    /// Artists with same name, but different capitalization
    ///
    /// See [`find::artist`]
    artists: Vec<Artist>,
}
/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistTemplate<'a> {
    /// Reference to the given artist
    artist: &'a Artist,
    /// This artist's playcount
    plays: usize,
    /// Percentage of this artist's plays to the total playcount
    percentage_of_plays: String,
    /// Time spent listening to this artist
    time_played: TimeDelta,
    /// Date of first artist entry
    first_listen: DateTime<Local>,
    /// Date of most recent artist entry
    last_listen: DateTime<Local>,
    /// This artist's ranking compared to other artists (playcount)
    position: usize,
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
#[expect(clippy::cast_precision_loss, reason = "necessary for % calc")]
#[expect(
    clippy::missing_panics_doc,
    reason = "unwraps which should never panic"
)]
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

    let entries = &state.entries;

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

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate { artists }.into_response();
    };

    let (plays, position) = *state.artists.get(artist).unwrap();
    let percentage_of_plays = format!(
        "{:.2}",
        (plays as f64 / gather::all_plays(entries) as f64) * 100.0
    );

    // unwrap ok bc already made sure artist exists earlier
    let first_listen = entries
        .iter()
        .find(|entry| artist.is_entry(entry))
        .unwrap()
        .timestamp;
    let last_listen = entries
        .iter()
        .rev()
        .find(|entry| artist.is_entry(entry))
        .unwrap()
        .timestamp;

    ArtistTemplate {
        plays,
        position,
        percentage_of_plays,
        time_played: gather::listening_time(entries, artist),
        first_listen,
        last_listen,
        artist,
    }
    .into_response()
}

/// GET `/artist/:artist_name(?id=usize)/absolute_plot`
///
/// Has to be in-lined in another base.html-derived template
pub async fn absolute_plot(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    options: Option<Query<ArtistQuery>>,
) -> Response {
    debug!(
        artist_name = artist_name,
        query = options.is_some(),
        "GET /artist/:artist_name(?query)/absolute_plot"
    );

    let entries = &state.entries;

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

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate { artists }.into_response();
    };

    // see endsong_ui::trace::absolute
    let mut times = Vec::<String>::with_capacity(entries.len());
    let mut plays = Vec::<usize>::with_capacity(entries.len());

    // since each date represents a single listen, we can just count up
    let mut artist_plays = 0;

    for entry in entries.iter().filter(|entry| artist.is_entry(entry)) {
        artist_plays += 1;
        times.push(entry.timestamp.format("%Y-%m-%d %H:%M").to_string());
        plays.push(artist_plays);
    }

    let trace = Scatter::new(times, plays).name(artist);

    let mut plot = Plot::new();
    plot.add_trace(trace);

    let layout = Layout::new()
        .template(plotly::layout::themes::PLOTLY_DARK.clone())
        .title(format!("{artist} | absolute plays"));
    plot.set_layout(layout);

    let plot_html = plot.to_inline_html(Some("artist-absolute-plot"));

    axum_extra::response::Html(plot_html).into_response()
}

/// GET `/artist/:artist_name(?id=usize)/relative_plot`
///
/// Has to be in-lined in another base.html-derived template
pub async fn relative_plot(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    options: Option<Query<ArtistQuery>>,
) -> Response {
    debug!(
        artist_name = artist_name,
        query = options.is_some(),
        "GET /artist/:artist_name(?query)/relative_plot"
    );

    let entries = &state.entries;

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

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate { artists }.into_response();
    };

    // see endsong_ui::trace::relative_to_all
    let mut times = Vec::<String>::with_capacity(entries.len());
    // percentages relative to the sum of all plays
    let mut plays = Vec::<f64>::with_capacity(entries.len());

    let mut artist_plays = 0.0;
    let mut all_plays = 0.0;

    // the plot should start at the first time the aspect is played
    let mut artist_found = false;

    for entry in entries.iter() {
        all_plays += 1.0;

        if artist.is_entry(entry) {
            artist_found = true;
            artist_plays += 1.0;
        }
        if artist_found {
            times.push(entry.timestamp.format("%Y-%m-%d %H:%M").to_string());
            // *100 so that the percentage is easier to read...
            plays.push(100.0 * (artist_plays / all_plays));
        }
    }

    let title = format!("{artist} | relative to all plays");
    let trace = Scatter::new(times, plays).name(title);

    let mut plot = Plot::new();
    plot.add_trace(trace);

    let layout = Layout::new()
        .template(plotly::layout::themes::PLOTLY_DARK.clone())
        .title(format!("{artist} | relative to all plays"));
    plot.set_layout(layout);

    let plot_html = plot.to_inline_html(Some("artist-relative-plot"));

    axum_extra::response::Html(plot_html).into_response()
}
