//! [![github]](https://github.com/fsktom/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files

// unsafe code is bad
#![deny(unsafe_code)]
// can be a pain, but it's worth it
// for stupid suggestions use #[allow(clippy::...)]
#![warn(clippy::pedantic)]
// because I want to be explicit when cloning is cheap
#![warn(clippy::clone_on_ref_ptr)]
// doc lints, checked when compiling/running clippy
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
// other doc lints, only checked when building docs
// https://doc.rust-lang.org/rustdoc/lints.html
// other good ones are warn by default
#![warn(rustdoc::missing_crate_level_docs, rustdoc::unescaped_backticks)]

pub mod plot;
pub mod print;
pub mod trace;
pub mod ui;

/// Creates a string with the given number of spaces
///
/// Returns an empty string if `num` <= 0 or `num` > 100
///
/// # Examples
/// ```
/// assert_eq!("   ", endsong_ui::spaces(3));
/// ```
#[must_use]
pub const fn spaces(num: usize) -> &'static str {
    endsong_macros::generate_spaces_match!(100)
}

/// Prelude containing all the modules,
/// a function for parsing dates, some structs used for printing,
/// and a trait to add a [pretty display method][print::DurationUtils::display]
/// to the [duration type][endsong::prelude::TimeDelta]
pub mod prelude {
    pub use crate::plot;
    pub use crate::print;
    pub use crate::trace;
    pub use crate::ui;

    pub use print::Aspect;
    pub use print::AspectFull;
    pub use print::DurationUtils;
    pub use print::Mode;
}
