//! [![github]](https://github.com/fsktom/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! Library for analyzing Spotify endsong.json files

pub mod find;
pub mod gather;
pub mod parse;
pub mod types;

pub use parse::LOCATION_TZ;
