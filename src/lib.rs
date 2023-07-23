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
pub mod parse;
pub mod types;

pub use parse::LOCATION_TZ;
