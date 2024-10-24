//! Contains templates for `/artist` routes

#![allow(clippy::module_name_repetitions, reason = "looks nicer")]

use crate::{not_found, AppState, ArtistInfo, UrlEncoding};

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Form,
};
use endsong::prelude::*;
use itertools::Itertools;
use plotly::{Layout, Plot, Scatter};
use rinja_axum::Template;
use serde::Deserialize;
use tracing::debug;

/// To choose an artist if there are multiple with same capitalization
/// (in my dataset tia)
#[derive(Deserialize)]
pub struct ArtistQuery {
    /// The artist's index in the [`Vec`] returned by [`find::artist`]
    artist_id: Option<usize>,
}

/// [`Template`] for if there are multiple artist with different
/// capitalization in [`base`]
#[derive(Template)]
#[template(path = "artist_selection.html", print = "none")]
pub struct ArtistSelectionTemplate {
    /// Artists with same name, but different capitalization
    ///
    /// See [`find::artist`]
    pub artists: Vec<Artist>,
    /// Link to the artist page (without `artist_id`)
    pub link_base_artist: String,
}
impl ArtistSelectionTemplate {
    /// Creates a new [`ArtistSelectionTemplate`] with generated `link_base_artist`
    #[expect(clippy::missing_panics_doc, reason = "unwrap will never panic")]
    #[must_use]
    pub fn new(artists: Vec<Artist>) -> Self {
        Self {
            link_base_artist: format!("/artist/{}", &artists.first().unwrap().encode()),
            artists,
        }
    }
}
/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "artist.html", print = "none")]
struct ArtistTemplate<'a> {
    /// Reference to the given artist
    artist: &'a Artist,
    /// Some info on the artist
    info: ArtistInfo,
    /// Percentage of this artist's plays to the total playcount
    percentage_of_plays: String,
    /// Percentage of this artist's playtime to the total time played
    percentage_of_duration: String,
    /// Date of first artist entry
    first_listen: DateTime<Local>,
    /// Date of most recent artist entry
    last_listen: DateTime<Local>,
    /// Link to albums
    link_albums: String,
    /// Link to songs
    link_songs: String,
    /// Link to absolute plot
    link_absolute: String,
    /// Link to relative plot
    link_relative: String,
}
/// GET `/artist/[:artist_name][?artist_id=usize]`
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
    Query(options): Query<ArtistQuery>,
) -> Response {
    debug!(
        artist_name = artist_name,
        artist_id = options.artist_id,
        "GET /artist/[:artist_name][?artist_id=usize]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate::new(artists).into_response();
    };

    let info = state.artist_info.get(artist).unwrap().clone();

    let percentage_of_plays = format!(
        "{:.2}",
        (info.plays as f64 / gather::all_plays(entries) as f64) * 100.0
    );
    let percentage_of_duration = format!(
        "{:.2}",
        (info.duration.num_seconds() as f64
            / gather::total_listening_time(entries).num_seconds() as f64)
            * 100.0
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

    let encoded_artist = artist.encode();
    let artist_id_arg = if let Some(artist_id) = options.artist_id {
        format!("?artist_id={artist_id}")
    } else {
        String::new()
    };
    let (link_albums, link_songs, link_absolute, link_relative) = (
        format!("/artist/{encoded_artist}/albums{artist_id_arg}"),
        format!("/artist/{encoded_artist}/songs{artist_id_arg}"),
        format!("/artist/{encoded_artist}/absolute_plot{artist_id_arg}"),
        format!("/artist/{encoded_artist}/relative_plot{artist_id_arg}"),
    );

    ArtistTemplate {
        artist,
        info,
        percentage_of_plays,
        percentage_of_duration,
        first_listen,
        last_listen,
        link_albums,
        link_songs,
        link_absolute,
        link_relative,
    }
    .into_response()
}

/// GET `/artist/[:artist_name]/absolute_lot[?artist_id=usize]`
///
/// Has to be in-lined in another base.html-derived template
pub async fn absolute_plot(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    Query(options): Query<ArtistQuery>,
) -> Response {
    debug!(
        artist_name = artist_name,
        artist_id = options.artist_id,
        "GET /artist/[:artist_name]/absolute_lot[?artist_id=usize]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate::new(artists).into_response();
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

/// GET `/artist/[:artist_name]/relative_plot[?artist_id=usize]`
///
/// Has to be in-lined in another base.html-derived template
pub async fn relative_plot(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    Query(options): Query<ArtistQuery>,
) -> Response {
    debug!(
        artist_name = artist_name,
        artist_id = options.artist_id,
        "GET /artist/[:artist_name]/relative_plot[?artist_id=usize]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate::new(artists).into_response();
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

/// Form for getting albums/songs
#[derive(Deserialize)]
pub struct ArtistForm {
    /// Amount of top albums/songs to return
    top: Option<usize>,
    /// Whether to sum song plays across albums
    ///
    /// Client sends either "on" or empty,
    /// so `Some(_) => true` and `None => false`
    sum_across_albums: Option<String>,
}

/// [`Template`] for [`albums`]
///
/// Has to be in-lined in another base.html-derived template
///
/// ```rinja
/// {% for (link, album, plays) in albums -%}
/// <li class="ml-7">
/// <a href="{{ link }}"
/// >{{ album.name }} | {{ plays }} plays</a
/// >
/// </li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct AlbumsTemplate {
    /// List of the artist's albums sorted by the playcount descending
    ///
    /// Elements: link to album page, [`Album`] instance, plays
    albums: Vec<(String, Album, usize)>,
}
/// POST `/artist/[:artist_name]/albums[?artist_id=usize][&top=usize]`
///
/// Lists of top albums
///
/// Has to be in-lined in another base.html-derived template
pub async fn albums(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    Query(options): Query<ArtistQuery>,
    Form(form): Form<ArtistForm>,
) -> Response {
    debug!(
        artist_name = artist_name,
        artist_id = options.artist_id,
        top = form.top,
        "POST /artist/[:artist_name]/albums[?artist_id=usize][&top=usize]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate::new(artists).into_response();
    };

    let top = form.top.unwrap_or(1000);

    let get_album_link = |album: &Album| {
        if let Some(artist_id) = options.artist_id {
            format!(
                "/album/{}/{}?artist_id={artist_id}",
                album.artist.encode(),
                album.encode()
            )
        } else {
            format!("/album/{}/{}", album.artist.encode(), album.encode())
        }
    };

    let album_map = gather::albums_from_artist(entries, artist);
    let albums = album_map
        .into_iter()
        .sorted_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
        .take(top)
        .map(|(album, plays)| (get_album_link(&album), album, plays))
        .collect();

    AlbumsTemplate { albums }.into_response()
}

/// [`Template`] for [`songs`]
///
/// Has to be in-lined in another base.html-derived template
///
/// ```rinja
/// {% for (link, song, plays) in songs -%}
/// <li class="ml-7">
/// <a href="{{ link }}"
/// >{{ song.name }} | {{ plays }} plays</a
/// >
/// </li>
/// {% endfor %}
/// ```
#[derive(Template)]
#[template(in_doc = true, ext = "html", print = "none")]
struct SongsTemplate {
    /// List of the artist's songs sorted by the playcount descending
    ///
    /// Elements: link to song page, [`Song`] instance, plays
    songs: Vec<(String, Song, usize)>,
}
/// POST `/artist/[:artist_name]/songs[?artist_id=usize][&top=usize][&sum_across_albums=String]`
///
/// Lists of top albums
///
/// Has to be in-lined in another base.html-derived template
pub async fn songs(
    State(state): State<Arc<AppState>>,
    Path(artist_name): Path<String>,
    Query(options): Query<ArtistQuery>,
    Form(form): Form<ArtistForm>,
) -> Response {
    debug!(
        artist_name = artist_name,
        artist_id = options.artist_id,
        top = form.top,
        sum_across_albums = form.sum_across_albums,
        "POST /artist/[:artist_name]/songs[?artist_id=usize][&top=usize][&sum_across_albums=String]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found().await.into_response();
    };

    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let Some(artist) = artist else {
        // query if multiple artists with different capitalization
        return ArtistSelectionTemplate::new(artists).into_response();
    };

    let top = form.top.unwrap_or(1000);

    let get_song_link = |song: &Song| {
        if let Some(artist_id) = options.artist_id {
            format!(
                "/song/{}/{}?artist_id={artist_id}",
                song.album.artist.encode(),
                song.encode()
            )
        } else {
            format!("/song/{}/{}", song.album.artist.encode(), song.encode())
        }
    };

    let song_map = if form.sum_across_albums.is_some() {
        gather::songs_from_artist_summed_across_albums(entries, artist)
    } else {
        gather::songs_from(entries, artist)
    };
    let songs = song_map
        .into_iter()
        .sorted_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
        .take(top)
        .map(|(song, plays)| (get_song_link(&song), song, plays))
        .collect();

    SongsTemplate { songs }.into_response()
}
