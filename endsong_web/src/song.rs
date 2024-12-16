//! Song page
//!
//! Contains handlers for `/song` routes

#![allow(clippy::module_name_repetitions, reason = "looks nicer")]

use crate::{
    artist::ArtistSelectionTemplate, encode_url, not_found_with_context, AppState, UrlEncoding,
};

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use endsong::prelude::*;
use rinja_axum::Template;
use serde::Deserialize;
use tracing::debug;

/// To choose an artist if there are multiple with same capitalization
#[derive(Deserialize)]
pub struct SongQuery {
    /// The artist's index in the [`Vec`] returned by [`find::artist`]
    artist_id: Option<usize>,
}

/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "song.html")]
struct SongTemplate {
    /// A song variant (to use for name + artist)
    base_song: Song,
    /// List of songs
    song_versions: Vec<(Song, usize)>,
    /// Link to artist page
    link_artist: String,
}
/// GET `/song/[:artist_name]/[:song_name][?artist_id=usize][&song_id=usize]`
///
/// # Panics
///
/// Shouldn't panic lol
#[expect(clippy::comparison_chain, reason = "couldn't bother")]
pub async fn base(
    State(state): State<Arc<AppState>>,
    Path((artist_name, song_name)): Path<(String, String)>,
    Query(options): Query<SongQuery>,
) -> impl IntoResponse {
    debug!(
        artist_name = artist_name,
        song_name = song_name,
        artist_id = options.artist_id,
        "GET /song/[:artist_name]/[:song_name][?artist_id=usize]"
    );

    let entries = &state.entries;

    let Some(artists) = entries.find().artist(&artist_name) else {
        return not_found_with_context(format!("An artist named {artist_name}"), "/", "home")
            .await
            .into_response();
    };

    // if there are multiple artists with same capitalizaton
    let artist = if artists.len() == 1 {
        artists.first()
    } else if let Some(artist_id) = options.artist_id {
        artists.get(artist_id)
    } else {
        None
    };

    let link_song_without_artist_id = format!(
        "/song/{}/{}",
        encode_url(&artist_name),
        encode_url(&song_name)
    );
    let link_base_artist = link_song_without_artist_id.clone();
    // mutliple artists + no artist_id was given in URL
    let Some(artist) = artist else {
        // => go to artist selection page
        return ArtistSelectionTemplate {
            artists,
            link_base_artist,
        }
        .into_response();
    };

    // create URL to the artist page (with the artist_id if it exists)
    let encoded_artist = artist.encode();
    let artist_id_arg = if let Some(artist_id) = options.artist_id {
        format!("?artist_id={artist_id}")
    } else {
        String::new()
    };
    let link_artist = format!("/artist/{encoded_artist}{artist_id_arg}");

    let Some(songs) = entries.find().song(&song_name, &artist_name) else {
        return not_found_with_context(
            format!("A song named {song_name} from {artist}"),
            &link_artist,
            "artist page",
        )
        .await
        .into_response();
    };

    // "base" song later on is supposed to be the one with the highest number of plays
    // (and first in alphabet if plays equal)
    let mut highest = (
        songs.first().unwrap().clone(),
        gather::plays(entries, songs.first().unwrap()),
    );

    // here we don't use song_id to differentiate between song capitalizations
    // unlike /artist with artist_id or /album with album_id
    // because we display all occurrences of this song across albums
    // wouldn't make sense to separate based on capitalization
    // so we also display occurrences across capitalizations

    let song_versions: Vec<(Song, usize)> = songs
        .iter()
        .filter(|song| song.album.artist == artist)
        .map(|song| {
            (song.clone(), {
                let plays = gather::plays(entries, song);
                // if same plays
                if plays == highest.1 {
                    // but earlier in alphabet
                    // (capitalization... ) => to make it deterministic
                    if song < &highest.0 {
                        // change
                        highest = (song.clone(), plays);
                    }
                // if higher plays
                } else if plays > highest.1 {
                    // change
                    highest = (song.clone(), plays);
                }
                plays
            })
        })
        .collect();

    // if song doesn't exist for given artist (but does for one with diff capitalization)
    if song_versions.is_empty() {
        return not_found_with_context(format!("A song named {song_name} from {artist}"), &link_song_without_artist_id, "artist selection page - it probably does exist for another artist with the same name, but capitalized differently!").await.into_response();
    }

    let base_song = highest.0;

    SongTemplate {
        base_song,
        song_versions,
        link_artist,
    }
    .into_response()
}
