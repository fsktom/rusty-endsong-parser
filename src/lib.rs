//! Library for getting data from Spotify endsong.json files

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

pub mod aspect;
pub mod entry;
pub mod find;
pub mod gather;

mod parse;

/// Re-exports the most commonly used items from this crate
/// and its dependencies.
pub mod prelude {
    pub use crate::{find, gather};

    pub use crate::entry::{SongEntries, SongEntry};

    pub use crate::aspect::{Album, Artist, Song};
    pub use crate::aspect::{HasSongs, Music};

    pub use crate::parse_date;

    // time and date related
    pub use chrono::{DateTime, Duration, Local, TimeZone};
}

use chrono::{DateTime, Local, TimeZone};
/// Converts a 'YYYY-MM-DD' string to a [`DateTime<Local>`]
/// in the context of the [`Local`] timezone
///
/// If you want more control (i.e. a certain hour/minute of the day)
/// use something like this instead:
/// ```
/// use endsong::prelude::*;
/// let date: DateTime<Local> = Local
///     .datetime_from_str("2020-06-03T01:01:01Z", "%FT%TZ")
///     .unwrap();
/// ```
/// See [`chrono::format::strftime`] for formatting details
///
/// # Arguments
///
/// `usr_input` - in YYYY-MM-DD format or 'now'/'end' or 'start'
/// - 'now'/'end' return the current time
/// - 'start' returns the start of UNIX epoch
///
/// # Examples
/// ```
/// use endsong::prelude::*;
///
/// let date: DateTime<Local> = parse_date("2020-06-03").unwrap();
/// assert_eq!(
///     date,
///     Local
///         .datetime_from_str("2020-06-03T00:00:00Z", "%FT%TZ")
///         .unwrap()
/// );
/// let unix_epoch: DateTime<Local> = parse_date("start").unwrap();
/// let now: DateTime<Local> = parse_date("now").unwrap();
/// ```
/// # Errors
///
/// Returns a [`ParseError`][chrono::format::ParseError]
/// if the `usr_input` cannot be parsed into a [`DateTime<Local>`]
#[allow(clippy::missing_panics_doc)]
pub fn parse_date(usr_input: &str) -> Result<DateTime<Local>, chrono::format::ParseError> {
    let date_str = match usr_input {
        "now" | "end" => return Ok(Local::now()),
        "start" => {
            let epoch = chrono::NaiveDateTime::from_timestamp_millis(0).unwrap();
            return Ok(Local.from_local_datetime(&epoch).unwrap());
        }
        // usr_input should be in YYYY-MM-DD format
        _ => format!("{usr_input}T00:00:00Z"),
    };

    // "%FT%TZ" is equivalent to "%Y-%m-%dT%H:%M:%SZ"
    // see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    Local.datetime_from_str(&date_str, "%FT%TZ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parser() {
        // correctly formatted input date
        assert_eq!(
            parse_date("2020-06-06").unwrap(),
            Local
                .datetime_from_str("2020-06-06T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
        );
        // https://users.rust-lang.org/t/idiomatic-way-of-testing-result-t-error/2171/4
        assert!(parse_date("2020-06-06").is_ok());

        // incorrectly formatted input date
        assert!(parse_date("feer3er3").is_err());
    }
}
