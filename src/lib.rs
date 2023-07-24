//! Library for getting data from Spotify endsong.json files

#![deny(unsafe_code)]
// To require working docs
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls
)]
#![warn(clippy::pedantic)]

pub mod find;
pub mod gather;
pub mod types;

pub use parse::LOCATION_TZ;

mod parse;

pub mod prelude {
    //! Re-exports the most commonly used items from this crate
    //! and its dependencies.
    pub use crate::{find, gather};

    pub use crate::types::{Album, Artist, Song};
    pub use crate::types::{SongEntries, SongEntry};

    // traits for dealing with Album, Artist and Song
    pub use crate::types::{HasArtist, HasSongs, Music};

    pub use crate::LOCATION_TZ;

    // time and date related
    pub use chrono::{DateTime, Duration, TimeZone};
    pub use chrono_tz::Tz;
}
