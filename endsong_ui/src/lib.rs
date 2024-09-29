//! [![github]](https://github.com/fsktom/rusty-endsong-parser/)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! CLI application with which you can analyze Spotify endsong.json files

// unsafe code is bad
#![deny(unsafe_code)]
// can be a pain, but it's worth it
// don't forget to use #[expect(clippy::...)] when sensible
#![warn(clippy::pedantic)]
// because I want to be explicit when cloning is cheap
#![warn(clippy::clone_on_ref_ptr)]
// doc lints, checked when compiling/running clippy
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
// other doc lints, only checked when building docs
// https://doc.rust-lang.org/rustdoc/lints.html
// other good ones are warn by default
#![warn(rustdoc::missing_crate_level_docs, rustdoc::unescaped_backticks)]
// https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html#expectlint
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::allow_attributes)]

pub mod plot;
pub mod print;
pub mod summarize;
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

/// Replaces Windows forbidden symbols in path with an '_'
///
/// Also removes whitespace and replaces empty
/// strings with '_'
#[must_use]
pub fn normalize_path(path: &str) -> String {
    // https://stackoverflow.com/a/31976060
    // Array > HashSet bc of overhead
    let forbidden_characters = [' ', '<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut new_path = String::with_capacity(path.len());

    for ch in path.chars() {
        if forbidden_characters.contains(&ch) {
            // replace a forbidden symbol with an underscore (for now...)
            new_path.push('_');
        } else {
            new_path.push(ch);
        }
    }

    // https://stackoverflow.com/a/1976050
    if new_path.is_empty() || new_path == "." {
        new_path = "_".into();
    }

    new_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_paths() {
        // should change the forbidden symbols to '_' in these
        assert_eq!(normalize_path("A|B"), "A_B");
        assert_eq!(normalize_path("A>B<C"), "A_B_C");
        assert_eq!(normalize_path(":A\"B"), "_A_B");
        assert_eq!(normalize_path("A/B"), normalize_path("A\\B"));
        assert_eq!(normalize_path("?A?"), "_A_");
        assert_eq!(normalize_path("A*B"), "A_B");

        // whitespace should be removed
        assert_eq!(normalize_path(" A"), "_A");
        assert_eq!(normalize_path("A "), "A_");
        assert_eq!(normalize_path(" "), "_");
        assert_eq!(normalize_path("   "), "___");

        // empty/only dot should be changed (Windows)
        assert_eq!(normalize_path(""), "_");
        assert_eq!(normalize_path("."), "_");

        assert_eq!(normalize_path(" A|B<>B? "), "_A_B__B__");

        // shouldn't change anything about these
        assert_eq!(normalize_path("A_B"), "A_B");
        assert_eq!(normalize_path("AB"), "AB");
    }
}

/// Prelude containing all the modules,
/// a function for parsing dates, some structs used for printing,
/// and a trait to add a [pretty display method][print::DurationUtils::display]
/// to the [duration type][endsong::prelude::TimeDelta]
pub mod prelude {
    pub use crate::plot;
    pub use crate::print;
    pub use crate::summarize;
    pub use crate::trace;
    pub use crate::ui;

    pub use print::Aspect;
    pub use print::DurationUtils;
    pub use print::Mode;

    pub use trace::TraceType;
}
