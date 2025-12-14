use core::{cmp, fmt, ops};
use std::iter::Peekable;
use std::str::{Chars, FromStr};

const EPOCH_YEAR: i32 = 2030;

#[derive(Debug, Copy, Clone)]
pub struct InvalidFieldError {
    pub field: &'static str,
    pub value: i32,
}

impl From<(&'static str, i32)> for InvalidFieldError {
    fn from((field, value): (&'static str, i32)) -> Self {
        InvalidFieldError { field, value }
    }
}

impl fmt::Display for InvalidFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid {}: {}", self.field, self.value)
    }
}

impl std::error::Error for InvalidFieldError {}

#[derive(Debug)]
pub enum ParseError {
    Parse(String),
    InvalidField(&'static str, i32),
    FormatMismatch,
}

impl From<InvalidFieldError> for ParseError {
    fn from(e: InvalidFieldError) -> Self {
        ParseError::InvalidField(e.field, e.value)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Parse(s) => write!(f, "parse error: {s}"),
            ParseError::InvalidField(field, value) => write!(f, "invalid {}: {}", field, value),
            ParseError::FormatMismatch => write!(f, "format mismatch"),
        }
    }
}

impl std::error::Error for ParseError {}

const fn month_days(year: i32) -> &'static [u32] {
    if is_leap_year(year) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    }
}

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const fn days_in_year(year: i32) -> i32 {
    if is_leap_year(year) { 366 } else { 365 }
}

/// A type representing a date and time.
/// It is represented by a `f64`, that is the seconds elapsed since Jan. 1, 2030, which is Eidoplot Epoch.
/// Timezone is not supported.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DateTime(f64);

impl DateTime {
    /// Build a DateTime from year, month and day
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, InvalidFieldError> {
        let date = DateComps { year, month, day };
        date.try_into()
    }

    /// The date time of the Eidoplot Epoch, 2030-01-01 00:00:00.
    pub const fn epoch() -> Self {
        DateTime(0.0)
    }

    /// The datetime of the Unix Epoch, 1970-01-01 00:00:00.
    pub const fn unix_epoch() -> Self {
        // seconds from Jan 1, 2030 to Jan 1, 1970
        DateTime(-1893456000.0)
    }

    /// Build a new datetime from a float timestamp.
    /// The value is in seconds elapsed since Jan 1, 2030.
    /// Returns None if the value is not a valid timestamp.
    /// (invalid timestamps are eg. NaN or Infinity)
    pub const fn from_timestamp(timestamp: f64) -> Option<Self> {
        if timestamp.is_finite() {
            Some(DateTime(timestamp))
        } else {
            None
        }
    }

    /// Get the internal representation as a float timestamp
    /// The value is in seconds elapsed since Jan 1, 2030 ([Self::epoch()]).
    /// (values before [Self::epoch()] are negative).
    ///
    /// The value is guaranteed to be a valid timestamp
    pub const fn timestamp(&self) -> f64 {
        self.0
    }

    /// Parse a string with the given format string.
    /// The format string supports the following specifiers:
    /// - `%Y` for year  (YYYY)
    /// - `%m` for month (MM)
    /// - `%d` for day   (DD)
    /// - `%H` for hour  (HH)
    /// - `%M` for minute (MM)
    /// - `%S` for second (SS)
    /// - `%.f` for second fraction (e.g. 340000 microseconds will format to ".34")
    /// - `%.3f` for milliseconds (e.g. 340000 microseconds will format to ".340")
    /// - `%.6f` for microseconds (e.g. 340000 microseconds will format to ".340000")
    /// - `%.9f` for nanoseconsd (e.g. 340000 microseconds will format to ".340000000")
    /// As a result, parsing according ISO 8601 can be done e.g. with `%Y-%m-%dT%H:%M:%S`
    pub fn fmt_parse(input: &str, fmt: &str) -> Result<DateTime, ParseError> {
        let comps = DateTimeComps::fmt_parse(input, fmt)?;
        Ok(comps.try_into()?)
    }

    /// Format this DateTime according to the given format string.
    /// See [DateTime::fmt_parse] for supported formats.
    pub fn fmt_write<W>(&self, fmt: &str, out: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.to_comps().fmt_write(fmt, out)
    }

    /// Format this DateTime according to the given format string.
    /// See [DateTime::fmt_parse] for supported formats.
    pub fn fmt_to_string(&self, fmt: &str) -> String {
        self.to_comps().fmt_to_string(fmt)
    }

    /// Compute the date components of this DateTime
    pub fn to_date(&self) -> DateComps {
        // days since epoch
        let mut days = (self.0 / 86400.0).floor() as i32;
        let mut year = EPOCH_YEAR;

        if days >= 0 {
            while days >= days_in_year(year) {
                days -= days_in_year(year);
                year += 1;
            }
        } else {
            while days < 0 {
                year -= 1;
                days += days_in_year(year);
            }
        }

        let month_days = month_days(year);
        let mut month = 0;
        while days >= month_days[month] as i32 {
            days -= month_days[month] as i32;
            month += 1;
        }

        DateComps {
            year,
            month: month as u32 + 1,
            day: days as u32 + 1,
        }
    }

    /// Compute the time components of this DateTime
    pub fn to_time(&self) -> TimeComps {
        let seconds_in_day = self.0 % 86400.0;
        let hour = (seconds_in_day / 3600.0).floor() as u32;
        let minute = ((seconds_in_day % 3600.0) / 60.0).floor() as u32;
        let second = (seconds_in_day % 60.0).floor() as u32;
        let micro = ((seconds_in_day % 1.0) * 1_000_000.0) as u32;

        TimeComps {
            hour,
            minute,
            second,
            micro,
        }
    }

    /// Compute the components of this DateTime
    pub fn to_comps(&self) -> DateTimeComps {
        let DateComps { year, month, day } = self.to_date();

        let TimeComps {
            hour,
            minute,
            second,
            micro,
        } = self.to_time();

        DateTimeComps {
            year,
            month,
            day,
            hour,
            minute,
            second,
            micro,
        }
    }
}

impl TryFrom<DateComps> for DateTime {
    type Error = InvalidFieldError;
    fn try_from(comps: DateComps) -> Result<DateTime, InvalidFieldError> {
        let comps = DateTimeComps {
            year: comps.year,
            month: comps.month,
            day: comps.day,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
        };
        DateTime::try_from(comps)
    }
}

/// Write the date time as a string with the format `%Y-%m-%d %H:%M:%S%.f`
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let comps = self.to_comps();
        if comps.micro == 0 {
            comps.fmt_write("%Y-%m-%d %H:%M:%S", f)
        } else {
            comps.fmt_write("%Y-%m-%d %H:%M:%S%.f", f)
        }
    }
}

/// A type gathering the date components
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateComps {
    /// The year (e.g. 2025)
    pub year: i32,
    /// The month (1 to 12)
    pub month: u32,
    /// The day in the month (1 to 31)
    pub day: u32,
}

/// A type gathering the date components
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeComps {
    /// The hour in the day (0 to 23)
    pub hour: u32,
    /// The minute in the hour (0 to 59)
    pub minute: u32,
    /// The second in the minute (0 to 59)
    pub second: u32,
    /// The microseconds in the second (0 to 999,999)
    pub micro: u32,
}

/// The components of a date time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTimeComps {
    /// The year (e.g. 2025)
    pub year: i32,
    /// The month (1 to 12)
    pub month: u32,
    /// The day in the month (1 to 31)
    pub day: u32,
    /// The hour in the day (0 to 23)
    pub hour: u32,
    /// The minute in the hour (0 to 59)
    pub minute: u32,
    /// The second in the minute (0 to 59)
    pub second: u32,
    /// The microseconds in the second (0 to 999,999)
    pub micro: u32,
}

impl DateTimeComps {
    /// The date time of the Eidoplot Epoch, 2030-01-01 00:00:00.
    pub const fn epoch() -> Self {
        DateTimeComps {
            year: EPOCH_YEAR,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
        }
    }

    /// The datetime of the Unix Epoch, 1970-01-01 00:00:00.
    pub const fn unix_epoch() -> Self {
        DateTimeComps {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
        }
    }

    /// Parse a string with the given format string.
    /// See [DateTime::fmt_parse] for supported formats.
    pub fn fmt_parse(input: &str, fmt: &str) -> Result<Self, ParseError> {
        let mut res = DateTimeComps {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
        };

        let mut input_chars = input.chars().peekable();

        let fmt = FmtStr(fmt);
        for tok in fmt.tokens() {
            let tok = tok?;
            match tok {
                FmtToken::Year => res.year = parse_number(&mut input_chars, 4)?,
                FmtToken::Month => res.month = parse_number(&mut input_chars, 2)?,
                FmtToken::Day => res.day = parse_number(&mut input_chars, 2)?,
                FmtToken::Hour => res.hour = parse_number(&mut input_chars, 2)?,
                FmtToken::Minute => res.minute = parse_number(&mut input_chars, 2)?,
                FmtToken::Second => res.second = parse_number(&mut input_chars, 2)?,
                FmtToken::Milli => res.micro = parse_fraction(&mut input_chars, Some(3))?,
                FmtToken::Micro => res.micro = parse_fraction(&mut input_chars, Some(6))?,
                FmtToken::Nano => res.micro = parse_fraction(&mut input_chars, Some(9))?,
                FmtToken::Frac => res.micro = parse_fraction(&mut input_chars, None)?,
                FmtToken::Lit(s) => {
                    for c in s.chars() {
                        if c != input_chars.next().ok_or(ParseError::FormatMismatch)? {
                            return Err(ParseError::FormatMismatch);
                        }
                    }
                }
                FmtToken::TimeDeltaDays => return Err(ParseError::FormatMismatch),
            }
        }

        // Validate all fields
        res.check_fields()?;

        Ok(res)
    }

    /// Format this DateTimeComps according to the given format string.
    /// See [DateTime::fmt_parse] for supported formats.
    pub fn fmt_write<W>(&self, fmt: &str, out: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let fmt = FmtStr(fmt);
        for tok in fmt.tokens() {
            let Ok(tok) = tok else { return Err(fmt::Error) };
            match tok {
                FmtToken::Year => write!(out, "{:04}", self.year)?,
                FmtToken::Month => write!(out, "{:02}", self.month)?,
                FmtToken::Day => write!(out, "{:02}", self.day)?,
                FmtToken::Hour => write!(out, "{:02}", self.hour)?,
                FmtToken::Minute => write!(out, "{:02}", self.minute)?,
                FmtToken::Second => write!(out, "{:02}", self.second)?,
                FmtToken::Milli => write!(out, ".{:03}", self.micro / 1000)?,
                FmtToken::Micro => write!(out, ".{:06}", self.micro)?,
                FmtToken::Nano => write!(out, ".{:09}", self.micro * 1000)?,
                FmtToken::Frac => format_micro_opt(out, self.micro)?,
                FmtToken::Lit(s) => out.write_str(s)?,
                FmtToken::TimeDeltaDays => return Err(fmt::Error),
            }
        }
        Ok(())
    }

    /// Format this DateTimeComps according to the given format string.
    /// See [DateTime::fmt_parse] for supported formats.
    pub fn fmt_to_string(&self, fmt: &str) -> String {
        let mut res = String::new();
        self.fmt_write(fmt, &mut res).unwrap();
        res
    }

    fn check_fields(&self) -> Result<(), InvalidFieldError> {
        if self.month < 1 || self.month > 12 {
            Err(("month", self.month as _).into())
        } else if self.day < 1 || self.day > month_days(self.year)[self.month as usize - 1] {
            Err(("day", self.day as _).into())
        } else if self.hour > 23 {
            Err(("hour", self.hour as _).into())
        } else if self.minute > 59 {
            Err(("minute", self.minute as _).into())
        } else if self.second > 59 {
            Err(("second", self.second as _).into())
        } else if self.micro > 999_999 {
            Err(("micro", self.micro as _).into())
        } else {
            Ok(())
        }
    }
}

impl TryFrom<DateTimeComps> for DateTime {
    type Error = InvalidFieldError;

    fn try_from(value: DateTimeComps) -> Result<Self, Self::Error> {
        value.check_fields()?;

        let mut days = value.day as i64 - 1;

        let month_days = month_days(value.year);
        // e.g. if month is march, we count days of January and February
        let mut month = value.month as usize - 1;
        while month != 0 {
            days += month_days[month - 1] as i64;
            month -= 1;
        }

        let mut year = value.year;
        while year > EPOCH_YEAR {
            year -= 1;
            days += days_in_year(year) as i64;
        }
        while year < EPOCH_YEAR {
            days -= days_in_year(year) as i64;
            year += 1;
        }

        let mut seconds = days as f64 * 86400.0;
        seconds += value.hour as f64 * 3600.0;
        seconds += value.minute as f64 * 60.0;
        seconds += value.second as f64;
        seconds += value.micro as f64 / 1_000_000.0;

        Ok(DateTime(seconds))
    }
}

impl From<DateTime> for DateTimeComps {
    fn from(value: DateTime) -> Self {
        value.to_comps()
    }
}

/// Write the date time as a string with the format `%Y-%m-%d %H:%M:%S%.f`
impl fmt::Display for DateTimeComps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_write("%Y-%m-%d %H:%M:%S%.f", f)
    }
}

/// A type representing a time difference, or duration.
/// The value can be negative.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeDelta(f64);

impl TimeDelta {
    pub const fn zero() -> Self {
        TimeDelta(0.0)
    }

    pub const fn from_days(days: f64) -> Self {
        TimeDelta(days * 86400.0)
    }

    pub const fn from_hours(hours: f64) -> Self {
        TimeDelta(hours * 3600.0)
    }

    pub const fn from_minutes(minutes: f64) -> Self {
        TimeDelta(minutes * 60.0)
    }

    /// Build a new time delta from a total number of seconds
    pub const fn from_seconds(seconds: f64) -> Self {
        TimeDelta(seconds)
    }

    /// Get the total duration in days
    pub const fn days(&self) -> f64 {
        self.0 / 86400.0
    }

    /// Get the total duration in hours
    pub const fn hours(&self) -> f64 {
        self.0 / 3600.0
    }

    /// Get the total duration in minutes
    pub const fn minutes(&self) -> f64 {
        self.0 / 60.0
    }

    /// Get the total duration in seconds
    pub const fn seconds(&self) -> f64 {
        self.0
    }

    /// Parse a string with the given format string.
    /// The format string supports the following specifiers:
    /// - `%D` for number of days followed by " day" or " days"
    ///    In parsing, both " day" and " days" are accepted.
    ///    In writing, it will write either " day" or " days" depending on the value
    ///    e.g. "1 day" or "3 days"
    /// - `%H` for hours, 24H wrapped
    /// - `%M` for minutes, 60M wrapped,
    /// - `%S` for second,s 60S wrapped,
    /// - `%.f` for second fraction (e.g. 340000 microseconds will format to ".34")
    /// - `%.3f` for milliseconds (e.g. 340000 microseconds will format to ".340")
    ///    maximum value is 999
    /// - `%.6f` for microseconds (e.g. 340000 microseconds will format to ".340000")
    ///    maximum value is 999999
    /// - `%.9f` for nanoseconsd (e.g. 340000 microseconds will format to ".340000000")
    ///    maximum value is 999999999
    ///    This will be floored to microsecond
    pub fn fmt_parse(input: &str, fmt: &str) -> Result<Self, ParseError> {
        let comps = TimeDeltaComps::fmt_parse(input, fmt)?;
        Ok(comps.try_into()?)
    }

    /// Formats a TimeDelta according to given string.
    /// See [TimeDelta::fmt_parse] for the supported specifiers.
    pub fn fmt_write<W>(&self, fmt: &str, out: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.to_comps().fmt_write(fmt, out)
    }

    /// Formats a TimeDelta according to given string.
    /// See [TimeDelta::fmt_parse] for the supported specifiers.
    pub fn fmt_to_string(&self, fmt: &str) -> String {
        self.to_comps().fmt_to_string(fmt)
    }

    /// Get the components of this time delta
    pub fn to_comps(&self) -> TimeDeltaComps {
        let total_secs = self.0.abs();
        let seconds_in_day = total_secs % 86400.0;

        let days = (total_secs / 86400.0).floor() as u32;
        let hours = (seconds_in_day / 3600.0).floor() as u32;
        let minutes = ((seconds_in_day % 3600.0) / 60.0).floor() as u32;
        let seconds = (seconds_in_day % 60.0).floor() as u32;
        let micro = ((seconds_in_day % 1.0) * 1_000_000.0) as u32;
        let is_neg = self.0 < 0.0;

        TimeDeltaComps {
            days,
            hour: hours,
            minute: minutes,
            second: seconds,
            micro,
            is_neg,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeDeltaComps {
    pub days: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub micro: u32,
    pub is_neg: bool,
}

impl TimeDeltaComps {
    pub const fn zero() -> Self {
        TimeDeltaComps {
            days: 0,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
            is_neg: false,
        }
    }

    /// Parse a string with the given format string.
    /// See [TimeDelta::fmt_parse] for the supported specifiers.
    pub fn fmt_parse(input: &str, fmt: &str) -> Result<Self, ParseError> {
        let mut res = TimeDeltaComps {
            days: 0,
            hour: 0,
            minute: 0,
            second: 0,
            micro: 0,
            is_neg: false,
        };

        let mut input_chars = input.chars().peekable();

        if input_chars.peek() == Some(&'-') {
            res.is_neg = true;
            input_chars.next();
        }

        let fmt = FmtStr(fmt);
        for tok in fmt.tokens() {
            let tok = tok?;
            match tok {
                FmtToken::TimeDeltaDays => {
                    res.days = parse_var_number(&mut input_chars)?;
                    // parse either " day" or " days"
                    for c in " day".chars() {
                        if c != input_chars.next().ok_or(ParseError::FormatMismatch)? {
                            return Err(ParseError::FormatMismatch);
                        }
                    }
                    if input_chars.peek() == Some(&'s') {
                        input_chars.next();
                    }
                }
                FmtToken::Hour => res.hour = parse_number(&mut input_chars, 2)?,
                FmtToken::Minute => res.minute = parse_number(&mut input_chars, 2)?,
                FmtToken::Second => res.second = parse_number(&mut input_chars, 2)?,
                FmtToken::Milli => res.micro = parse_fraction(&mut input_chars, Some(3))?,
                FmtToken::Micro => res.micro = parse_fraction(&mut input_chars, Some(6))?,
                FmtToken::Nano => res.micro = parse_fraction(&mut input_chars, Some(9))?,
                FmtToken::Frac => res.micro = parse_fraction(&mut input_chars, None)?,
                FmtToken::Lit(s) => {
                    for c in s.chars() {
                        if c != input_chars.next().ok_or(ParseError::FormatMismatch)? {
                            return Err(ParseError::FormatMismatch);
                        }
                    }
                }
                _ => return Err(ParseError::FormatMismatch),
            }
        }

        // Validate all fields
        res.check_fields()?;

        Ok(res)
    }

    /// Formats a TimeDeltaComps according to given string.
    /// See [TimeDelta::fmt_parse] for the supported specifiers.
    pub fn fmt_write<W>(&self, fmt: &str, out: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        if self.is_neg {
            write!(out, "-")?;
        }
        let fmt = FmtStr(fmt);
        for tok in fmt.tokens() {
            let Ok(tok) = tok else { return Err(fmt::Error) };
            match tok {
                FmtToken::TimeDeltaDays => write!(
                    out,
                    "{} day{}",
                    self.days,
                    if self.days > 1 { "s" } else { "" }
                )?,
                FmtToken::Hour => write!(out, "{:02}", self.hour)?,
                FmtToken::Minute => write!(out, "{:02}", self.minute)?,
                FmtToken::Second => write!(out, "{:02}", self.second)?,
                FmtToken::Milli => write!(out, ".{:03}", self.micro / 1000)?,
                FmtToken::Micro => write!(out, ".{:06}", self.micro)?,
                FmtToken::Nano => write!(out, ".{:09}", self.micro * 1000)?,
                FmtToken::Frac => format_micro_opt(out, self.micro)?,
                FmtToken::Lit(s) => out.write_str(s)?,
                _ => return Err(fmt::Error),
            }
        }
        Ok(())
    }

    /// Formats a TimeDeltaComps according to given string.
    /// See [TimeDelta::fmt_parse] for the supported specifiers.
    pub fn fmt_to_string(&self, fmt: &str) -> String {
        let mut res = String::new();
        self.fmt_write(fmt, &mut res).unwrap();
        res
    }

    pub fn check_fields(&self) -> Result<(), InvalidFieldError> {
        if self.hour > 23 {
            Err(("hour", self.hour as _).into())
        } else if self.minute > 59 {
            Err(("minute", self.minute as _).into())
        } else if self.second > 59 {
            Err(("second", self.second as _).into())
        } else if self.micro > 999_999 {
            Err(("micro", self.micro as _).into())
        } else {
            Ok(())
        }
    }
}

impl TryFrom<TimeDeltaComps> for TimeDelta {
    type Error = InvalidFieldError;

    fn try_from(value: TimeDeltaComps) -> Result<Self, Self::Error> {
        value.check_fields()?;

        let total_seconds = value.days as f64 * 86400.0
            + value.hour as f64 * 3600.0
            + value.minute as f64 * 60.0
            + value.second as f64
            + value.micro as f64 / 1_000_000.0;

        if value.is_neg {
            Ok(TimeDelta(-total_seconds))
        } else {
            Ok(TimeDelta(total_seconds))
        }
    }
}

impl fmt::Display for TimeDelta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_comps().fmt(f)
    }
}

impl fmt::Display for TimeDeltaComps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt = if self.days > 0 {
            "%D %H:%M:%S"
        } else {
            "%H:%M:%S"
        };
        self.fmt_write(fmt, f)
    }
}

impl cmp::Eq for DateTime {}

impl cmp::Ord for DateTime {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ops::Sub<DateTime> for DateTime {
    type Output = TimeDelta;
    fn sub(self, rhs: DateTime) -> TimeDelta {
        TimeDelta(self.0 - rhs.0)
    }
}

impl ops::Sub<TimeDelta> for DateTime {
    type Output = DateTime;
    fn sub(self, rhs: TimeDelta) -> DateTime {
        DateTime(self.0 - rhs.0)
    }
}

impl ops::Add<TimeDelta> for DateTime {
    type Output = DateTime;
    fn add(self, rhs: TimeDelta) -> DateTime {
        DateTime(self.0 + rhs.0)
    }
}

impl ops::AddAssign<TimeDelta> for DateTime {
    fn add_assign(&mut self, rhs: TimeDelta) {
        self.0 += rhs.0;
    }
}

impl ops::SubAssign<TimeDelta> for DateTime {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        self.0 -= rhs.0;
    }
}

impl cmp::Eq for TimeDelta {}

impl cmp::Ord for TimeDelta {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ops::Add<TimeDelta> for TimeDelta {
    type Output = TimeDelta;
    fn add(self, rhs: TimeDelta) -> TimeDelta {
        TimeDelta(self.0 + rhs.0)
    }
}

impl ops::Sub<TimeDelta> for TimeDelta {
    type Output = TimeDelta;
    fn sub(self, rhs: TimeDelta) -> TimeDelta {
        TimeDelta(self.0 - rhs.0)
    }
}

impl ops::Mul<f64> for TimeDelta {
    type Output = TimeDelta;
    fn mul(self, rhs: f64) -> TimeDelta {
        TimeDelta(self.0 * rhs)
    }
}

impl ops::Div<f64> for TimeDelta {
    type Output = TimeDelta;
    fn div(self, rhs: f64) -> TimeDelta {
        TimeDelta(self.0 / rhs)
    }
}

impl ops::AddAssign<TimeDelta> for TimeDelta {
    fn add_assign(&mut self, rhs: TimeDelta) {
        self.0 += rhs.0;
    }
}

impl ops::SubAssign<TimeDelta> for TimeDelta {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        self.0 -= rhs.0;
    }
}

#[derive(Debug, Clone, Copy)]
struct FmtStr<'a>(&'a str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FmtToken<'a> {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Milli,
    Micro,
    Nano,
    Frac,
    Lit(&'a str),
    TimeDeltaDays,
}

impl FmtStr<'_> {
    fn tokens(&self) -> FmtTokens<'_> {
        FmtTokens { remaining: self.0 }
    }
}

#[derive(Debug, Clone)]
struct FmtTokens<'a> {
    remaining: &'a str,
}

impl<'a> Iterator for FmtTokens<'a> {
    type Item = Result<FmtToken<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        if self.remaining.len() >= 4 {
            match &self.remaining[..4] {
                "%.3f" => {
                    self.remaining = &self.remaining[4..];
                    return Some(Ok(FmtToken::Milli));
                }
                "%.6f" => {
                    self.remaining = &self.remaining[4..];
                    return Some(Ok(FmtToken::Micro));
                }
                "%.9f" => {
                    self.remaining = &self.remaining[4..];
                    return Some(Ok(FmtToken::Nano));
                }
                _ => (),
            }
        }

        if self.remaining.len() >= 3 {
            match &self.remaining[..3] {
                "%.f" => {
                    self.remaining = &self.remaining[3..];
                    return Some(Ok(FmtToken::Frac));
                }
                _ => (),
            }
        }

        if self.remaining.len() >= 2 {
            match &self.remaining[..2] {
                "%Y" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Year));
                }
                "%m" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Month));
                }
                "%d" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Day));
                }
                "%H" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Hour));
                }
                "%M" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Minute));
                }
                "%S" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Second));
                }
                "%%" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::Lit("%")));
                }
                "%D" => {
                    self.remaining = &self.remaining[2..];
                    return Some(Ok(FmtToken::TimeDeltaDays));
                }
                _ => (),
            }
        }

        if self.remaining.chars().next() == Some('%') {
            self.remaining = &self.remaining[1..];
            return Some(Err(ParseError::FormatMismatch));
        }

        let mut end = 0;
        for (i, c) in self.remaining.char_indices() {
            end = i;
            if c == '%' {
                break;
            }
        }
        if end == 0 {
            None
        } else {
            let lit = &self.remaining[..end];
            self.remaining = &self.remaining[end..];
            Some(Ok(FmtToken::Lit(lit)))
        }
    }
}

/// Parse a fixed-width number from the input
fn parse_number<T: FromStr>(chars: &mut Peekable<Chars>, width: usize) -> Result<T, ParseError>
where
    T::Err: fmt::Debug,
{
    let mut s = String::with_capacity(width);
    for _ in 0..width {
        if let Some(c) = chars.next() {
            s.push(c);
        } else {
            return Err(ParseError::Parse("Unexpected end of input".to_string()));
        }
    }
    s.parse()
        .map_err(|_| ParseError::Parse("Failed to parse number".to_string()))
}

/// Parse a variable-width number from the input
fn parse_var_number<T: FromStr>(chars: &mut Peekable<Chars>) -> Result<T, ParseError>
where
    T::Err: fmt::Debug,
{
    let mut s = String::new();
    loop {
        if let Some(c) = chars.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            s.push(*c);
            chars.next();
        } else {
            return Err(ParseError::Parse("Unexpected end of input".to_string()));
        }
    }
    s.parse()
        .map_err(|_| ParseError::Parse("Failed to parse number".to_string()))
}

/// Parse the fractional seconds (microseconds)
fn parse_fraction(chars: &mut Peekable<Chars>, len: Option<usize>) -> Result<u32, ParseError> {
    // Skip the '.'
    if chars.next() != Some('.') {
        return Err(ParseError::FormatMismatch);
    }
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            s.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    if let Some(len) = len {
        if s.len() != len {
            return Err(ParseError::FormatMismatch);
        }
    }

    while s.len() < 6 {
        s.push('0');
    }
    let s = &s[..6];

    let micro = s.parse().unwrap_or(0);
    Ok(micro)
}

fn format_micro_opt<W: fmt::Write>(out: &mut W, mut micro: u32) -> fmt::Result {
    if micro != 0 {
        write!(out, ".")?;

        let mut factors = [100_000, 10_000, 1000, 100, 10].into_iter();
        while micro != 0 {
            let Some(f) = factors.next() else { break };
            if micro >= f {
                write!(out, "{}", micro / f)?;
                micro %= f;
            } else {
                write!(out, "0")?;
            }
        }
        if micro > 0 {
            // µs units
            write!(out, "{}", micro)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_tokens() {
        let fmt = FmtStr("%Y-%m-%d %H:%M:%S%.f");
        let tokens: Vec<FmtToken> = fmt.tokens().map(|res| res.unwrap()).collect();
        assert_eq!(
            tokens,
            vec![
                FmtToken::Year,
                FmtToken::Lit("-"),
                FmtToken::Month,
                FmtToken::Lit("-"),
                FmtToken::Day,
                FmtToken::Lit(" "),
                FmtToken::Hour,
                FmtToken::Lit(":"),
                FmtToken::Minute,
                FmtToken::Lit(":"),
                FmtToken::Second,
                FmtToken::Frac,
            ]
        );
    }

    #[test]
    fn test_parse_datetime_comps() {
        let input = "2025-01-13 15:46:32.25";
        let fmt = "%Y-%m-%d %H:%M:%S%.f";
        let result = DateTimeComps::fmt_parse(input, fmt).unwrap();
        assert_eq!(result.year, 2025);
        assert_eq!(result.month, 1);
        assert_eq!(result.day, 13);
        assert_eq!(result.hour, 15);
        assert_eq!(result.minute, 46);
        assert_eq!(result.second, 32);
        assert_eq!(result.micro, 250000);
    }

    #[test]
    fn test_parse_datetime_comps_invalid_month() {
        let input = "2025-13-01 15:46:32";
        let fmt = "%Y-%m-%d %H:%M:%S";
        let result = DateTimeComps::fmt_parse(input, fmt);
        assert!(matches!(result, Err(ParseError::InvalidField("month", 13))));
    }

    #[test]
    fn test_parse_datetime_comps_microseconds_truncated() {
        let input = "2025-01-13 15:46:32.123456789";
        let fmt = "%Y-%m-%d %H:%M:%S%.f";
        let result = DateTimeComps::fmt_parse(input, fmt).unwrap();
        assert_eq!(result.year, 2025);
        assert_eq!(result.month, 1);
        assert_eq!(result.day, 13);
        assert_eq!(result.hour, 15);
        assert_eq!(result.minute, 46);
        assert_eq!(result.second, 32);
        assert_eq!(result.micro, 123456);
    }

    #[test]
    fn test_parse_datetime_comps_no_usecs() {
        let input = "2025-01-13 15:46:32";
        let fmt = "%Y-%m-%d %H:%M:%S%.f";
        let result = DateTimeComps::fmt_parse(input, fmt);
        assert!(matches!(result, Err(ParseError::FormatMismatch)));
    }

    #[test]
    fn test_comps_to_string() {
        let comps = DateTimeComps {
            year: 2025,
            month: 1,
            day: 13,
            hour: 15,
            minute: 46,
            second: 32,
            micro: 250000,
        };
        let result = comps.to_string();
        assert_eq!(result, "2025-01-13 15:46:32.25");
    }

    #[test]
    fn test_format_comps() {
        let comps = DateTimeComps {
            year: 2025,
            month: 1,
            day: 13,
            hour: 15,
            minute: 46,
            second: 32,
            micro: 250000,
        };
        let fmt = "%Y-%m-%d %H:%M:%S%.6f";
        let result = comps.fmt_to_string(fmt);
        assert_eq!(result, "2025-01-13 15:46:32.250000");
    }
}
