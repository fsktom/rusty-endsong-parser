//! Contains template for /album routes

#![allow(clippy::module_name_repetitions, reason = "looks nicer")]

use crate::{artist::ArtistSelectionTemplate, encode_url, not_found, AppState, UrlEncoding};

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use endsong::prelude::*;
use rinja_axum::Template;
use serde::Deserialize;
use tracing::debug;

/// To choose an artist and album if there are multiple with same capitalization
#[derive(Deserialize)]
pub struct AlbumQuery {
    /// The artist's index in the [`Vec`] returned by [`find::artist`]
    artist_id: Option<usize>,
    /// The albums's index in the [`Vec`] returned by [`find::album`]
    album_id: Option<usize>,
}

/// [`Template`] for if there are multiple artist with different
/// capitalization in [`base`]
#[derive(Template)]
#[template(path = "album_selection.html", print = "none")]
struct AlbumSelectionTemplate {
    /// Albums with same name, but different capitalization
    ///
    /// Will only happen if you didn't do [`SongEntries::sum_different_capitalization`]
    ///
    /// See [`find::album`]
    albums: Vec<Album>,
    /// Link to the album page (without `album_id`)
    link_base_album: String,
}
/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "album.html", print = "none")]
struct AlbumTemplate<'a> {
    /// Reference to the given Album
    album: &'a Album,
    /// This album's playcount
    plays: usize,
    /// Percentage of this album's plays to the total playcount
    percentage_of_plays: String,
    /// Percentage of this album's plays to the artist playcount
    percentage_of_artist_plays: String,
    /// Time spent listening to this artist
    time_played: TimeDelta,
    /// Date of first artist entry
    first_listen: DateTime<Local>,
    /// Date of most recent artist entry
    last_listen: DateTime<Local>,
    /// Link to artist page
    link_artist: String,
}
/// GET `/album/[:artist_name]/[:album_name][?artist_id=usize][?album_id=usize]`
///
/// Artist page
///
/// Returns an [`AlbumTemplate`] with a valid `artist_name` and `album_name`,
/// an [`ArtistSelectionTemplate`] if there are
/// multiple artists with this name
/// but different capitalization,
/// an [`AlbumSelectionTemplate`] if there are
/// multiple artists with this name
/// but different capitalization,
/// and [`not_found`] if the artist or album is not in the dataset
#[expect(clippy::cast_precision_loss, reason = "necessary for % calc")]
#[expect(
    clippy::missing_panics_doc,
    reason = "unwraps which should never panic"
)]
pub async fn base(
    State(state): State<Arc<AppState>>,
    Path((artist_name, album_name)): Path<(String, String)>,
    Query(options): Query<AlbumQuery>,
) -> Response {
    debug!(
        artist_name = artist_name,
        album_name = album_name,
        artist_id = options.artist_id,
        album_id = options.album_id,
        "GET /album/[:artist_name]/[:album_name][?artist_id=usize][?album_id=usize]"
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
        return ArtistSelectionTemplate {
            link_base_artist: format!(
                "/album/{}/{}",
                encode_url(&artist_name),
                encode_url(&album_name)
            ),
            artists,
        }
        .into_response();
    };

    let Some(albums) = entries.find().album(&album_name, &artist_name) else {
        return not_found().await.into_response();
    };

    let album = if albums.len() == 1 {
        albums.first()
    } else if let Some(album_id) = options.album_id {
        albums.get(album_id)
    } else {
        None
    };

    let encoded_artist = artist.encode();

    let Some(album) = album else {
        // unwrap ok - find().album() guaranteed to contain at least one album if Some, see earlier
        let encoded_album = albums.first().unwrap().encode();

        let link_base_album = if let Some(artist_id) = options.artist_id {
            format!("/album/{encoded_artist}/{encoded_album}?artist_id={artist_id}")
        } else {
            format!("/album/{encoded_artist}/{encoded_album}")
        };

        return AlbumSelectionTemplate {
            albums,
            link_base_album,
        }
        .into_response();
    };

    // http://localhost:3000/album/TiA/%E6%B5%81%E6%98%9F
    // (i.e. could only happen on manual link entry)
    if album.artist != artist {
        return Redirect::permanent(&format!(
            "/artist/{encoded_artist}?artist_id={}",
            // unwrap ok - you're right after the artist sel page if this happens -> artist_id exists
            options.artist_id.unwrap()
        ))
        .into_response();
    }

    let plays = gather::plays(entries, album);
    let percentage_of_plays = format!(
        "{:.2}",
        (plays as f64 / gather::all_plays(entries) as f64) * 100.0
    );
    let percentage_of_artist_plays = format!(
        "{:.2}",
        (plays as f64 / gather::plays(entries, artist) as f64) * 100.0
    );

    // unwrap ok bc already made sure artist exists earlier
    let first_listen = gather::first_entry_of(entries, album).unwrap().timestamp;
    let last_listen = gather::last_entry_of(entries, album).unwrap().timestamp;

    let link_artist = if let Some(artist_id) = options.artist_id {
        format!("/artist/{encoded_artist}?artist_id={artist_id}")
    } else {
        format!("/artist/{encoded_artist}")
    };

    AlbumTemplate {
        plays,
        percentage_of_plays,
        percentage_of_artist_plays,
        time_played: gather::listening_time(entries, album),
        first_listen,
        last_listen,
        link_artist,
        album,
    }
    .into_response()
}
