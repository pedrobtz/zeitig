//! Helpers to move whole columns between R vectors and `jiff` values.
//!
//! R owns recycling, so every function here assumes its input columns already
//! have a common length (checked by `common_len()`). `NA` in any field of a
//! record marks the element as missing and is written back as `NA` in every
//! output field.

use std::borrow::Cow;
use std::rc::Rc;

use jiff::civil::{Date, DateTime, Time};
use jiff::{Span, Timestamp, Zoned};
use savvy::{
    IntegerSexp, ListSexp, OwnedIntegerSexp, OwnedListSexp, OwnedRealSexp, OwnedStringSexp,
    RealSexp, Sexp, StringSexp, TypedSexp,
};

use crate::tz::TzCache;

/// R's `NA_integer_`. Compared directly instead of through savvy's
/// `NotAvailableValue`, which reads the `R_NaInt` symbol and so cannot be
/// linked into `cargo test` binaries.
pub(crate) const NA_INT: i32 = i32::MIN;

pub(crate) fn is_na_int(x: i32) -> bool {
    x == NA_INT
}

/// `NA_character_` check for strings from a `StringSexp`.
pub(crate) fn is_na_str(x: &str) -> bool {
    use savvy::NotAvailableValue;
    x.is_na()
}

/// Error for element `i` (0-based), with the 1-based index R users see.
pub(crate) fn elt_error(i: usize, e: impl std::fmt::Display) -> savvy::Error {
    let msg = e.to_string();
    savvy::Error::new(format!("{} (element {})", temporal_message(&msg), i + 1))
}

/// Rewrites the few jiff error messages that name Rust items (`jiff::Span`,
/// `SpanRelativeTo::days_are_24_hours()`, ...) in terms of zeitig's API, as
/// design.md section 6 promises Temporal wording. Only runs on the error
/// path.
pub(crate) fn temporal_message(msg: &str) -> Cow<'_, str> {
    if !msg.contains("jiff::") && !msg.contains("relative reference time") {
        return Cow::Borrowed(msg);
    }
    const PHRASES: [(&str, &str); 9] = [
        (
            " (operations on `jiff::Timestamp`, `jiff::tz::Offset` and `jiff::civil::Time` \
             don't support calendar units in a `jiff::Span`)",
            " (instants and plain times have no calendar, so years, months, weeks and days \
             are not allowed)",
        ),
        (
            "requires that either a relative reference time be given or \
             `jiff::SpanRelativeTo::days_are_24_hours()` is used to indicate invariant \
             24-hour days, but neither were provided",
            "requires `relative_to`",
        ),
        (
            "requires that a relative reference time be given \
             (`jiff::SpanRelativeTo::days_are_24_hours()` was given but this only permits \
             using days and weeks without a relative reference time)",
            "requires `relative_to` (without it, only days of 24 hours and weeks of 7 days \
             can be used)",
        ),
        (
            "requires that a relative reference time be given, but none was provided",
            "requires `relative_to`",
        ),
        (
            ", parse as a `jiff::Timestamp` first and convert to a civil date/time instead",
            "; parse it with instant() or zoned_date_time() and convert the result instead",
        ),
        (
            " (perhaps try parsing into a `jiff::Span` instead)",
            " (parse it with duration() instead)",
        ),
        (" (must use `jiff::Span::to_duration` instead)", ""),
        ("numeric `jiff::tz::Offset`", "numeric UTC offset"),
        ("relative reference time", "`relative_to` value"),
    ];
    const NAMES: [(&str, &str); 9] = [
        ("`jiff::civil::DateTime`", "plain date-time"),
        ("`jiff::civil::Date`", "plain date"),
        ("`jiff::civil::Time`", "plain time"),
        ("`jiff::Timestamp`", "instant"),
        ("`jiff::Zoned`", "zoned date-time"),
        ("`jiff::Span`", "duration"),
        ("`jiff::SignedDuration`", "exact duration"),
        ("`jiff::tz::Offset`", "UTC offset"),
        (
            "`jiff::SpanRelativeTo::days_are_24_hours()`",
            "24-hour days",
        ),
    ];
    let mut out = msg.to_string();
    for (from, to) in PHRASES.iter().chain(NAMES.iter()) {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    Cow::Owned(out)
}

/// Common length of a set of columns; errors if they differ (a bug in the R
/// caller, which must recycle first).
pub(crate) fn common_len(lens: &[usize]) -> savvy::Result<usize> {
    let n = lens.first().copied().unwrap_or(0);
    if lens.iter().any(|&l| l != n) {
        return Err(savvy::Error::new(
            "internal error: columns of different lengths",
        ));
    }
    Ok(n)
}

/// Builds a named R list from integer columns.
pub(crate) fn int_list(names: &[&str], cols: Vec<Vec<i32>>) -> savvy::Result<Sexp> {
    let mut out = OwnedListSexp::new(names.len(), true)?;
    for (k, (name, col)) in names.iter().zip(cols).enumerate() {
        let v = OwnedIntegerSexp::try_from_slice(col)?;
        out.set_name_and_value(k, name, v)?;
    }
    Ok(out.into())
}

fn to_i8(v: i32) -> Option<i8> {
    i8::try_from(v).ok()
}

fn to_i16(v: i32) -> Option<i16> {
    i16::try_from(v).ok()
}

// ---------------------------------------------------------------------------
// Record fields passed as a list. The handles below are cheap (no copy); the
// readers built from them borrow the R vectors' data.

/// The first `N` fields of a record as integer vectors.
pub(crate) fn int_cols<const N: usize>(x: &ListSexp) -> savvy::Result<[IntegerSexp; N]> {
    let mut cols = Vec::with_capacity(N);
    for k in 0..N {
        match x.get_by_index(k).map(|s| s.into_typed()) {
            Some(TypedSexp::Integer(v)) => cols.push(v),
            _ => {
                return Err(savvy::Error::new(format!(
                    "internal error: record field {} must be an integer vector",
                    k + 1
                )))
            }
        }
    }
    cols.try_into()
        .map_err(|_| savvy::Error::new("internal error: wrong number of record fields"))
}

pub(crate) fn real_col(x: &ListSexp, k: usize) -> savvy::Result<RealSexp> {
    match x.get_by_index(k).map(|s| s.into_typed()) {
        Some(TypedSexp::Real(v)) => Ok(v),
        _ => Err(savvy::Error::new(format!(
            "internal error: record field {} must be a double vector",
            k + 1
        ))),
    }
}

pub(crate) fn int_col(x: &ListSexp, k: usize) -> savvy::Result<IntegerSexp> {
    match x.get_by_index(k).map(|s| s.into_typed()) {
        Some(TypedSexp::Integer(v)) => Ok(v),
        _ => Err(savvy::Error::new(format!(
            "internal error: record field {} must be an integer vector",
            k + 1
        ))),
    }
}

pub(crate) fn str_col(x: &ListSexp, k: usize) -> savvy::Result<StringSexp> {
    match x.get_by_index(k).map(|s| s.into_typed()) {
        Some(TypedSexp::String(v)) => Ok(v),
        _ => Err(savvy::Error::new(format!(
            "internal error: record field {} must be a character vector",
            k + 1
        ))),
    }
}

/// Strings of a character vector, `None` for `NA`. The `&'static str`s point
/// into R's global string cache, so this allocates one `Vec`, not a string
/// per element.
pub(crate) fn str_values(x: &StringSexp) -> Vec<Option<&'static str>> {
    x.iter()
        .map(|s| if is_na_str(s) { None } else { Some(s) })
        .collect()
}

// ---------------------------------------------------------------------------
// PlainDate: year, month, day

/// The fields of a plain date record.
pub(crate) struct DateCols([IntegerSexp; 3]);

impl DateCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        Ok(Self(int_cols::<3>(x)?))
    }

    pub(crate) fn reader(&self) -> savvy::Result<DateIn<'_>> {
        let c = &self.0;
        DateIn::new(&c[0], &c[1], &c[2])
    }
}

pub(crate) struct DateIn<'a> {
    y: &'a [i32],
    m: &'a [i32],
    d: &'a [i32],
}

impl<'a> DateIn<'a> {
    pub(crate) fn new(
        y: &'a IntegerSexp,
        m: &'a IntegerSexp,
        d: &'a IntegerSexp,
    ) -> savvy::Result<Self> {
        common_len(&[y.len(), m.len(), d.len()])?;
        Ok(Self {
            y: y.as_slice(),
            m: m.as_slice(),
            d: d.as_slice(),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.y.len()
    }

    /// Element `i` as a `Date`, `None` when missing. Stored values are always
    /// valid (they were produced by Rust), so a failure here means a record
    /// was modified by hand; it is reported as an error, never a panic.
    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<Date>> {
        let (y, m, d) = (self.y[i], self.m[i], self.d[i]);
        if is_na_int(y) || is_na_int(m) || is_na_int(d) {
            return Ok(None);
        }
        let date = match (to_i16(y), to_i8(m), to_i8(d)) {
            (Some(y), Some(m), Some(d)) => Date::new(y, m, d).map_err(|e| elt_error(i, e))?,
            _ => return Err(elt_error(i, "invalid date fields")),
        };
        Ok(Some(date))
    }
}

#[derive(Default)]
pub(crate) struct DateOut {
    y: Vec<i32>,
    m: Vec<i32>,
    d: Vec<i32>,
}

impl DateOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            y: Vec::with_capacity(n),
            m: Vec::with_capacity(n),
            d: Vec::with_capacity(n),
        }
    }

    pub(crate) fn push(&mut self, x: Option<Date>) {
        match x {
            Some(x) => {
                self.y.push(x.year().into());
                self.m.push(x.month().into());
                self.d.push(x.day().into());
            }
            None => {
                self.y.push(NA_INT);
                self.m.push(NA_INT);
                self.d.push(NA_INT);
            }
        }
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        int_list(&["year", "month", "day"], vec![self.y, self.m, self.d])
    }
}

// ---------------------------------------------------------------------------
// PlainTime: second_of_day, nanos

/// The fields of a plain time record.
pub(crate) struct TimeCols([IntegerSexp; 2]);

impl TimeCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        Ok(Self(int_cols::<2>(x)?))
    }

    pub(crate) fn reader(&self) -> savvy::Result<TimeIn<'_>> {
        TimeIn::new(&self.0[0], &self.0[1])
    }
}

pub(crate) struct TimeIn<'a> {
    sod: &'a [i32],
    ns: &'a [i32],
}

impl<'a> TimeIn<'a> {
    pub(crate) fn new(sod: &'a IntegerSexp, ns: &'a IntegerSexp) -> savvy::Result<Self> {
        common_len(&[sod.len(), ns.len()])?;
        Ok(Self {
            sod: sod.as_slice(),
            ns: ns.as_slice(),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.sod.len()
    }

    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<Time>> {
        let (sod, ns) = (self.sod[i], self.ns[i]);
        if is_na_int(sod) || is_na_int(ns) {
            return Ok(None);
        }
        if !(0..86_400).contains(&sod) || !(0..1_000_000_000).contains(&ns) {
            return Err(elt_error(i, "invalid time fields"));
        }
        let t = Time::new(
            (sod / 3600) as i8,
            (sod / 60 % 60) as i8,
            (sod % 60) as i8,
            ns,
        )
        .map_err(|e| elt_error(i, e))?;
        Ok(Some(t))
    }
}

#[derive(Default)]
pub(crate) struct TimeOut {
    sod: Vec<i32>,
    ns: Vec<i32>,
}

pub(crate) fn second_of_day(t: Time) -> i32 {
    i32::from(t.hour()) * 3600 + i32::from(t.minute()) * 60 + i32::from(t.second())
}

impl TimeOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            sod: Vec::with_capacity(n),
            ns: Vec::with_capacity(n),
        }
    }

    pub(crate) fn push(&mut self, x: Option<Time>) {
        match x {
            Some(t) => {
                self.sod.push(second_of_day(t));
                self.ns.push(t.subsec_nanosecond());
            }
            None => {
                self.sod.push(NA_INT);
                self.ns.push(NA_INT);
            }
        }
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        int_list(&["second_of_day", "nanos"], vec![self.sod, self.ns])
    }
}

// ---------------------------------------------------------------------------
// PlainDateTime: year, month, day, second_of_day, nanos

/// The fields of a plain date-time record.
pub(crate) struct DateTimeCols([IntegerSexp; 5]);

impl DateTimeCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        Ok(Self(int_cols::<5>(x)?))
    }

    pub(crate) fn reader(&self) -> savvy::Result<DateTimeIn<'_>> {
        let c = &self.0;
        DateTimeIn::new(&c[0], &c[1], &c[2], &c[3], &c[4])
    }
}

pub(crate) struct DateTimeIn<'a> {
    date: DateIn<'a>,
    time: TimeIn<'a>,
}

impl<'a> DateTimeIn<'a> {
    pub(crate) fn new(
        y: &'a IntegerSexp,
        m: &'a IntegerSexp,
        d: &'a IntegerSexp,
        sod: &'a IntegerSexp,
        ns: &'a IntegerSexp,
    ) -> savvy::Result<Self> {
        common_len(&[y.len(), sod.len()])?;
        Ok(Self {
            date: DateIn::new(y, m, d)?,
            time: TimeIn::new(sod, ns)?,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.date.len()
    }

    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<DateTime>> {
        match (self.date.get(i)?, self.time.get(i)?) {
            (Some(d), Some(t)) => Ok(Some(d.to_datetime(t))),
            _ => Ok(None),
        }
    }
}

#[derive(Default)]
pub(crate) struct DateTimeOut {
    date: DateOut,
    time: TimeOut,
}

impl DateTimeOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            date: DateOut::with_capacity(n),
            time: TimeOut::with_capacity(n),
        }
    }

    pub(crate) fn push(&mut self, x: Option<DateTime>) {
        self.date.push(x.map(|x| x.date()));
        self.time.push(x.map(|x| x.time()));
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        int_list(
            &["year", "month", "day", "second_of_day", "nanos"],
            vec![
                self.date.y,
                self.date.m,
                self.date.d,
                self.time.sod,
                self.time.ns,
            ],
        )
    }
}

// ---------------------------------------------------------------------------
// Duration: years, months, weeks, days, hours, minutes, seconds,
// milliseconds, microseconds, nanoseconds (all doubles holding integers)

pub(crate) const DURATION_FIELDS: [&str; 10] = [
    "years",
    "months",
    "weeks",
    "days",
    "hours",
    "minutes",
    "seconds",
    "milliseconds",
    "microseconds",
    "nanoseconds",
];

/// Builds a `Span` from Temporal duration fields. `None` when any field is
/// `NA`; an error for non-integers, mixed signs or values outside jiff's
/// per-unit limits. One pass over the fields, no allocation, and zero fields
/// skip jiff's per-unit setter.
pub(crate) fn span_from_fields(i: usize, v: [f64; 10]) -> savvy::Result<Option<Span>> {
    if v.iter().any(|x| x.is_nan()) {
        return Ok(None);
    }
    let mut sign = 0.0;
    let mut a = [0i64; 10];
    for (k, &x) in v.iter().enumerate() {
        if x == 0.0 {
            continue;
        }
        if !x.is_finite() || x.fract() != 0.0 {
            return Err(elt_error(
                i,
                format!(
                    "duration field '{}' must be a finite integer",
                    DURATION_FIELDS[k]
                ),
            ));
        }
        if sign != 0.0 && x.signum() != sign {
            return Err(elt_error(
                i,
                "mixed-sign values not allowed as duration fields",
            ));
        }
        sign = x.signum();
        let m = x.abs();
        if m >= 9.223_372_036_854_775e18 {
            return Err(elt_error(
                i,
                format!("duration field '{}' is out of range", DURATION_FIELDS[k]),
            ));
        }
        a[k] = m as i64;
    }
    if sign == 0.0 {
        return Ok(Some(Span::new()));
    }
    let e = |e| elt_error(i, e);
    let mut span = Span::new();
    for (k, &n) in a.iter().enumerate() {
        if n == 0 {
            continue;
        }
        span = match k {
            0 => span.try_years(n),
            1 => span.try_months(n),
            2 => span.try_weeks(n),
            3 => span.try_days(n),
            4 => span.try_hours(n),
            5 => span.try_minutes(n),
            6 => span.try_seconds(n),
            7 => span.try_milliseconds(n),
            8 => span.try_microseconds(n),
            _ => span.try_nanoseconds(n),
        }
        .map_err(e)?;
    }
    Ok(Some(if sign < 0.0 { span.negate() } else { span }))
}

pub(crate) fn span_to_fields(s: Span) -> [f64; 10] {
    [
        f64::from(s.get_years()),
        f64::from(s.get_months()),
        f64::from(s.get_weeks()),
        f64::from(s.get_days()),
        f64::from(s.get_hours()),
        s.get_minutes() as f64,
        s.get_seconds() as f64,
        s.get_milliseconds() as f64,
        s.get_microseconds() as f64,
        s.get_nanoseconds() as f64,
    ]
}

/// The ten fields of a duration record, in `DURATION_FIELDS` order.
pub(crate) struct DurationCols(Vec<RealSexp>);

impl DurationCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        let mut cols = Vec::with_capacity(10);
        for (k, name) in DURATION_FIELDS.iter().enumerate() {
            match x.get_by_index(k).map(|s| s.into_typed()) {
                Some(TypedSexp::Real(r)) => cols.push(r),
                _ => {
                    return Err(savvy::Error::new(format!(
                        "internal error: duration field '{name}' must be a double vector"
                    )))
                }
            }
        }
        Ok(Self(cols))
    }

    pub(crate) fn reader(&self) -> savvy::Result<DurationIn<'_>> {
        let cols: [&[f64]; 10] = std::array::from_fn(|k| self.0[k].as_slice());
        let lens: [usize; 10] = cols.map(|c| c.len());
        let n = common_len(&lens)?;
        Ok(DurationIn { cols, n })
    }
}

pub(crate) struct DurationIn<'a> {
    cols: [&'a [f64]; 10],
    n: usize,
}

impl DurationIn<'_> {
    pub(crate) fn len(&self) -> usize {
        self.n
    }

    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<Span>> {
        span_from_fields(i, self.cols.map(|c| c[i]))
    }
}

pub(crate) struct DurationOut {
    cols: [Vec<f64>; 10],
}

impl DurationOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            cols: std::array::from_fn(|_| Vec::with_capacity(n)),
        }
    }

    pub(crate) fn push(&mut self, x: Option<Span>) {
        match x {
            Some(s) => {
                for (c, v) in self.cols.iter_mut().zip(span_to_fields(s)) {
                    // Normalise -0 to 0.
                    c.push(v + 0.0);
                }
            }
            None => {
                for c in self.cols.iter_mut() {
                    c.push(na_real());
                }
            }
        }
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        let mut out = OwnedListSexp::new(10, true)?;
        for (k, col) in self.cols.into_iter().enumerate() {
            out.set_name_and_value(k, DURATION_FIELDS[k], OwnedRealSexp::try_from_slice(col)?)?;
        }
        Ok(out.into())
    }
}

/// R's `NA_real_`: a NaN with payload 1954. A function rather than a const
/// because `f64::from_bits` is only `const` from Rust 1.83 (MSRV is 1.81).
pub(crate) fn na_real() -> f64 {
    f64::from_bits(0x7FF0_0000_0000_07A2)
}

// ---------------------------------------------------------------------------
// Instant: seconds (double), nanos (integer)

/// The fields of an instant record (also the first two of a zoned one).
pub(crate) struct InstantCols {
    secs: RealSexp,
    nanos: IntegerSexp,
}

impl InstantCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        Ok(Self {
            secs: real_col(x, 0)?,
            nanos: int_col(x, 1)?,
        })
    }

    pub(crate) fn reader(&self) -> savvy::Result<InstantIn<'_>> {
        let (secs, nanos) = (self.secs.as_slice(), self.nanos.as_slice());
        common_len(&[secs.len(), nanos.len()])?;
        Ok(InstantIn { secs, nanos })
    }
}

pub(crate) struct InstantIn<'a> {
    secs: &'a [f64],
    nanos: &'a [i32],
}

impl InstantIn<'_> {
    pub(crate) fn len(&self) -> usize {
        self.secs.len()
    }

    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<Timestamp>> {
        let (s, ns) = (self.secs[i], self.nanos[i]);
        if s.is_nan() || is_na_int(ns) {
            return Ok(None);
        }
        timestamp_from_parts(i, s, ns).map(Some)
    }
}

pub(crate) fn timestamp_from_parts(i: usize, secs: f64, nanos: i32) -> savvy::Result<Timestamp> {
    if !secs.is_finite() || secs.fract() != 0.0 || secs.abs() > 1e15 {
        return Err(elt_error(
            i,
            "epoch seconds must be a finite integer in range",
        ));
    }
    Timestamp::new(secs as i64, nanos).map_err(|e| elt_error(i, e))
}

pub(crate) struct InstantOut {
    secs: Vec<f64>,
    nanos: Vec<i32>,
}

/// Splits a timestamp into whole seconds (floored) and nanoseconds in
/// `0..1e9`, so that record order matches time order.
pub(crate) fn timestamp_parts(t: Timestamp) -> (f64, i32) {
    let mut s = t.as_second();
    let mut ns = t.subsec_nanosecond();
    if ns < 0 {
        s -= 1;
        ns += 1_000_000_000;
    }
    (s as f64, ns)
}

impl InstantOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            secs: Vec::with_capacity(n),
            nanos: Vec::with_capacity(n),
        }
    }

    pub(crate) fn push(&mut self, x: Option<Timestamp>) {
        match x {
            Some(t) => {
                let (s, ns) = timestamp_parts(t);
                self.secs.push(s);
                self.nanos.push(ns);
            }
            None => {
                self.secs.push(na_real());
                self.nanos.push(NA_INT);
            }
        }
    }

    fn write_into(self, out: &mut OwnedListSexp) -> savvy::Result<()> {
        out.set_name_and_value(0, "seconds", OwnedRealSexp::try_from_slice(self.secs)?)?;
        out.set_name_and_value(1, "nanos", OwnedIntegerSexp::try_from_slice(self.nanos)?)?;
        Ok(())
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        let mut out = OwnedListSexp::new(2, true)?;
        self.write_into(&mut out)?;
        Ok(out.into())
    }
}

// ---------------------------------------------------------------------------
// ZonedDateTime: seconds, nanos, tz

/// The fields of a zoned date-time record.
pub(crate) struct ZonedCols {
    inst: InstantCols,
    tz: StringSexp,
}

impl ZonedCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        Ok(Self {
            inst: InstantCols::new(x)?,
            tz: str_col(x, 2)?,
        })
    }

    pub(crate) fn reader(&self) -> savvy::Result<ZonedIn<'_>> {
        let inst = self.inst.reader()?;
        let tz = str_values(&self.tz);
        common_len(&[inst.len(), tz.len()])?;
        Ok(ZonedIn { inst, tz })
    }
}

pub(crate) struct ZonedIn<'a> {
    inst: InstantIn<'a>,
    tz: Vec<Option<&'static str>>,
}

impl ZonedIn<'_> {
    pub(crate) fn len(&self) -> usize {
        self.inst.len()
    }

    /// Element `i` and its canonical time zone identifier, `None` when
    /// missing.
    pub(crate) fn get(
        &self,
        i: usize,
        cache: &mut TzCache,
    ) -> savvy::Result<Option<(Zoned, Rc<str>)>> {
        let (Some(t), Some(id)) = (self.inst.get(i)?, self.tz[i]) else {
            return Ok(None);
        };
        let r = cache.get(i, id)?;
        Ok(Some((t.to_zoned(r.tz.clone()), r.id.clone())))
    }
}

/// Zoned date-time columns being built. Time zone identifiers are shared
/// `Rc<str>`s from the `TzCache`, so pushing an element allocates nothing.
pub(crate) struct ZonedOut {
    inst: InstantOut,
    tz: Vec<Option<Rc<str>>>,
}

impl ZonedOut {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            inst: InstantOut::with_capacity(n),
            tz: Vec::with_capacity(n),
        }
    }

    /// Pushes a zoned date-time and the canonical identifier of its zone.
    pub(crate) fn push(&mut self, x: Option<(&Zoned, &Rc<str>)>) {
        match x {
            Some((z, id)) => {
                self.inst.push(Some(z.timestamp()));
                self.tz.push(Some(id.clone()));
            }
            None => {
                self.inst.push(None);
                self.tz.push(None);
            }
        }
    }

    pub(crate) fn into_sexp(self) -> savvy::Result<Sexp> {
        let mut out = OwnedListSexp::new(3, true)?;
        self.inst.write_into(&mut out)?;
        let mut tz = OwnedStringSexp::new(self.tz.len())?;
        for (i, v) in self.tz.iter().enumerate() {
            match v {
                Some(s) => tz.set_elt(i, s)?,
                None => tz.set_na(i)?,
            }
        }
        out.set_name_and_value(2, "tz", tz)?;
        Ok(out.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jiff_message(r: Result<impl std::fmt::Debug, jiff::Error>) -> String {
        r.unwrap_err().to_string()
    }

    #[test]
    fn jiff_messages_are_rewritten() {
        use jiff::{SpanRelativeTo, ToSpan, Unit};
        let calendar = jiff_message(Timestamp::UNIX_EPOCH.checked_add(1.day()));
        assert!(calendar.contains("jiff::"), "{calendar}");
        let msg = temporal_message(&calendar);
        assert!(!msg.contains("jiff::"), "{msg}");
        assert!(msg.contains("have no calendar"), "{msg}");

        let months = jiff_message(
            1.month()
                .total((Unit::Day, SpanRelativeTo::days_are_24_hours())),
        );
        let msg = temporal_message(&months);
        assert!(!msg.contains("jiff::"), "{msg}");
        assert!(msg.contains("requires `relative_to`"), "{msg}");

        let zulu = jiff_message("2020-01-01T00:00Z".parse::<Date>());
        let msg = temporal_message(&zulu);
        assert!(!msg.contains("jiff::"), "{msg}");

        let plain = "parameter 'day' with value 31 is not in the required range of 1..=30";
        assert!(matches!(temporal_message(plain), Cow::Borrowed(_)));
    }

    #[test]
    fn span_fields_skip_zeros_and_check_signs() {
        let v = [0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, -5.0];
        let span = span_from_fields(0, v).unwrap().unwrap();
        assert_eq!(span_to_fields(span), v);
        let zero = span_from_fields(0, [0.0; 10]).unwrap().unwrap();
        assert!(zero.is_zero());
        let mut inf = [0.0; 10];
        inf[3] = f64::INFINITY;
        assert!(span_from_fields(0, inf).is_err());
    }
}
