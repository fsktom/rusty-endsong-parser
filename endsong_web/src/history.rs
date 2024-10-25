//! Contains templates for `/history` routes

use crate::AppState;

use std::sync::Arc;

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{MappedLocalTime, NaiveDate, NaiveTime};
use endsong::prelude::*;
use itertools::Itertools;
use rinja::Template;
use serde::{Deserialize, Deserializer};
use tracing::debug;

/// [`Template`] for [`base`]
#[derive(Template)]
#[template(path = "history.html", print = "none")]
struct BaseTemplate;
/// GET `/history`
pub async fn base() -> impl IntoResponse {
    debug!("GET /history");

    BaseTemplate {}
}

/// Form for stuff used in [`elements`]
#[derive(Deserialize)]
#[expect(clippy::module_name_repetitions, reason = "looks good")]
pub struct HistoryForm {
    /// First date to look for history
    start_date: Option<String>,
    /// Last date to look for history
    end_date: Option<String>,
}
struct Element {
    /// Date of entry
    timestamp: DateTime<Local>,
    /// Duration of entry
    time_played: TimeDelta,
    /// [`Song`] instance
    song: Song,
}
/// Template for table with entries in the given period
#[derive(Template)]
#[template(path = "history_elements.html", print = "none")]
struct ElementsTemplate {
    /// Filtered out entries with initialized [`Song`] structs for convenience
    entries: Vec<Element>,
}
/// POST `/history[?start_date=String][&end_date=String]`
pub async fn elements(
    State(state): State<Arc<AppState>>,
    Form(form): Form<HistoryForm>,
) -> impl IntoResponse {
    debug!(
        start_date = form.start_date,
        end_date = form.end_date,
        "POST /history[?start_date=String][&end_date=String]"
    );

    let entries = &state.entries;

    let Some(dates) = parse_dates(entries, form.start_date, form.end_date) else {
        return (
            StatusCode::BAD_REQUEST,
            axum_extra::response::Html("<p>Invalid dates!</p>"),
        )
            .into_response();
    };
    let (start_date, end_date) = dates;
    if end_date <= start_date {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum_extra::response::Html("<p>Invalid dates - start date is before end date!</p>"),
        )
            .into_response();
    }
    let filtered_entries = entries.between(&start_date, &end_date);

    if filtered_entries.is_empty() {
        return axum_extra::response::Html(
            "<p>You haven't listened to anything during this period!</p>",
        )
        .into_response();
    }

    let elements = filtered_entries
        .iter()
        .map(|e| Element {
            timestamp: e.timestamp,
            time_played: e.time_played,
            song: Song::from(e),
        })
        .collect_vec();

    ElementsTemplate { entries: elements }.into_response()
}

/// Parses dates in `YYYY-MM-DD` format and returns (`first_date`, `last_date`) tuple
fn parse_dates(
    entries: &SongEntries,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Option<(DateTime<Local>, DateTime<Local>)> {
    let first_date = entries.first_date();
    let last_date = entries.last_date();

    let zero_time = NaiveTime::from_hms_opt(0, 0, 0)?;

    let start_date = start_date?;
    let Ok(start_date) = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d") else {
        return None;
    };
    let MappedLocalTime::Single(start_date) =
        start_date.and_time(zero_time).and_local_timezone(Local)
    else {
        return None;
    };
    let start_date = if start_date < first_date {
        first_date
    } else {
        start_date
    };

    let Some(end_date) = end_date else {
        // if no end_date given, it's set to the full day of start_date
        let end_date = start_date + TimeDelta::days(1);
        return Some((start_date, end_date));
    };

    let Ok(end_date) = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d") else {
        return None;
    };
    let MappedLocalTime::Single(end_date) = end_date.and_time(zero_time).and_local_timezone(Local)
    else {
        return None;
    };
    let end_date = if end_date > last_date {
        last_date
    } else {
        end_date
    };

    Some((start_date, end_date))
}

/// Models the date picker mode
pub enum DatePickerMode {
    /// One date picker for a single day
    Single,
    /// Two date pickers for a time period
    TimePeriod,
}
impl std::fmt::Display for DatePickerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single => write!(f, "single"),
            Self::TimePeriod => write!(f, "time period"),
        }
    }
}
impl<'de> Deserialize<'de> for DatePickerMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "period" => Ok(Self::TimePeriod),
            _ => Ok(Self::Single),
        }
    }
}

/// Form used by [`datepicker`]
#[derive(Deserialize)]
pub struct DatePickerForm {
    /// Whether to show a single date picker or two
    mode: DatePickerMode,
}
/// [`Template`] used in [`datepicker`]
#[derive(Template)]
#[template(path = "history_datepicker.html", print = "none")]
struct DatePickerTemplate {
    /// Whether to display single date picker or two
    single: bool,
    /// Dataset's first day of listening
    first: DateTime<Local>,
    /// First day of listening plus one day
    first_plus_one: DateTime<Local>,
    /// Most recent day of listening
    last: DateTime<Local>,
}
/// POST `/history/datepicker`
pub async fn date_picker(
    State(state): State<Arc<AppState>>,
    Form(form): Form<DatePickerForm>,
) -> impl IntoResponse {
    let entries = &state.entries;

    let single = match form.mode {
        DatePickerMode::Single => true,
        DatePickerMode::TimePeriod => false,
    };

    let first = entries.first_date();
    let first_plus_one = entries.first_date() + TimeDelta::days(1);
    let last = entries.last_date();

    DatePickerTemplate {
        single,
        first,
        first_plus_one,
        last,
    }
}

/// Filters in use by `history_datepicker.html`
mod filters {
    #![allow(clippy::unnecessary_wraps, reason = "rinja required output type")]
    use endsong::prelude::*;

    use crate::UrlEncoding;

    /// Formats a date in `YYYY-MM-DD` format to use in HTML datepicker
    pub fn date_basic(date: &DateTime<Local>) -> rinja::Result<String> {
        Ok(date.format("%Y-%m-%d").to_string())
    }

    /// Pretty formats a [`TimeDelta`] in a reasonable way
    ///
    /// ```"_m _s"```
    pub fn pretty_duration(duration: &TimeDelta) -> rinja::Result<String> {
        let seconds = duration.num_seconds();
        let minutes = duration.num_minutes();

        if minutes == 0 {
            return Ok(format!("{seconds}s"));
        }

        let remaining_seconds = seconds % 60;

        Ok(format!("{minutes}m {remaining_seconds}s"))
    }

    /// Creates a link to the given [`Artist`]
    pub fn link_artist(artist: &Artist) -> rinja::Result<String> {
        Ok(format!("/artist/{}", artist.encode()))
    }

    /// Creates a link to the given [`Album`]
    pub fn link_album(album: &Album) -> rinja::Result<String> {
        Ok(format!(
            "/album/{}/{}",
            album.artist.encode(),
            album.encode()
        ))
    }

    /// Creates a link to the given [`Song`]
    pub fn link_song(song: &Song) -> rinja::Result<String> {
        Ok(format!(
            "/song/{}/{}",
            song.album.artist.encode(),
            song.encode()
        ))
    }
}
