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
/// Converts a `YYYY-MM-DD` string to a [`DateTime<Local>`]
/// in the context of the [`Local`] timezone
///
/// If you want more control (i.e. a certain hour/minute of the day)
/// use something like this instead:
/// ```
/// use endsong::prelude::*;
/// let date: DateTime<Local> = Local
///     .datetime_from_str("2020-06-03T01:01:01Z", "%FT%TZ")?;
/// # Ok::<(), chrono::format::ParseError>(())
/// ```
/// See [`chrono::format::strftime`] for formatting details
///
/// or maybe:
/// ```
/// use endsong::prelude::*;
/// let date: DateTime<Local> = Local.with_ymd_and_hms(2020, 6, 3, 1, 1, 1).unwrap();
/// ```
///
/// # Arguments
///
/// `date` - in YYYY-MM-DD format or 'now'/'end' or 'start'
/// - 'now'/'end' return the current time
/// - 'start' returns the start of UNIX epoch
///
/// # Examples
/// ```
/// use endsong::prelude::*;
///
/// let date: DateTime<Local> = parse_date("2020-06-03")?;
/// assert_eq!(
///     date,
///     Local.datetime_from_str("2020-06-03T00:00:00Z", "%FT%TZ")?
/// );
///
/// let unix_epoch: DateTime<Local> = parse_date("start")?;
/// assert_eq!(
///     unix_epoch,
///     chrono::Utc.datetime_from_str("1970-01-01T00:00:00Z", "%FT%TZ")?
/// );
///
/// let now: DateTime<Local> = parse_date("now")?;
/// # Ok::<(), chrono::format::ParseError>(())
/// ```
/// # Errors
///
/// Returns a [`ParseError`][chrono::format::ParseError]
/// if the `date` does not follow the format `YYYY-MM-DD`
/// and is not 'now'/'end'/'start'
pub fn parse_date(date: &str) -> Result<DateTime<Local>, chrono::format::ParseError> {
    match date {
        "now" | "end" => Ok(Local::now()),
        "start" => {
            let epoch = chrono::Utc.datetime_from_str("1970-01-01T00:00:00Z", "%FT%TZ")?;
            Ok(Local.from_utc_datetime(&epoch.naive_utc()))
        }
        // date should in YYYY-MM-DD format
        _ => Local.datetime_from_str(&format!("{date}T00:00:00Z"), "%FT%TZ"),
    }
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
        assert_eq!(
            parse_date("2021-12-13").unwrap(),
            Local.with_ymd_and_hms(2021, 12, 13, 0, 0, 0).unwrap()
        );

        // valid input dates
        assert!(parse_date("2020-12-06").is_ok());
        assert!(parse_date("0000-12-06").is_ok());
        assert!(parse_date("9000-12-06").is_ok());
        assert!(parse_date("2024-02-29").is_ok());

        // special keyword values
        assert!(parse_date("now").is_ok());
        assert!(parse_date("end").is_ok());
        assert_eq!(
            parse_date("start").unwrap(),
            chrono::Utc
                .datetime_from_str("1970-01-01T00:00:00Z", "%FT%TZ")
                .unwrap()
        );

        // incorrectly formatted input date
        assert!(parse_date("").is_err());
        assert!(parse_date(" ").is_err());
        assert!(parse_date("2011/01/01").is_err());
        assert!(parse_date("01/01/2011").is_err());
        assert!(parse_date("01.01.2011").is_err());
        assert!(parse_date("2020-06-06T00:00:00Z").is_err());

        // invalid input date
        assert!(parse_date("2011-00-00").is_err());
        assert!(parse_date("2011-00-01").is_err());
        assert!(parse_date("2011-01-00").is_err());
        assert!(parse_date("2011-13-12").is_err());
        assert!(parse_date("2023-02-29").is_err());

        // for some reason, the chrono parser accepts leading whitespace
        // but not trailing whitespace...
        assert!(parse_date("  2011-01-01").is_ok());
        assert!(parse_date("2011-01-01 ").is_err());
        assert!(parse_date(" 2011-01-01 ").is_err());
    }
}
