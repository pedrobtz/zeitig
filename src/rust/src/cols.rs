//! Helpers to move whole columns between R vectors and `jiff` values.
//!
//! R owns recycling, so every function here assumes its input columns already
//! have a common length (checked by `common_len()`). `NA` in any field of a
//! record marks the element as missing and is written back as `NA` in every
//! output field.

use jiff::civil::{Date, DateTime, Time};
use jiff::Span;
use savvy::{
    IntegerSexp, ListSexp, OwnedIntegerSexp, OwnedListSexp, OwnedRealSexp, Sexp, TypedSexp,
};

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
    savvy::Error::new(format!("{e} (element {})", i + 1))
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
// PlainDate: year, month, day

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
/// per-unit limits.
pub(crate) fn span_from_fields(i: usize, v: [f64; 10]) -> savvy::Result<Option<Span>> {
    if v.iter().any(|x| x.is_nan()) {
        return Ok(None);
    }
    let mut sign = 0.0;
    for (k, &x) in v.iter().enumerate() {
        if !x.is_finite() || x.fract() != 0.0 {
            return Err(elt_error(
                i,
                format!(
                    "duration field '{}' must be a finite integer",
                    DURATION_FIELDS[k]
                ),
            ));
        }
        if x != 0.0 {
            if sign != 0.0 && x.signum() != sign {
                return Err(elt_error(
                    i,
                    "mixed-sign values not allowed as duration fields",
                ));
            }
            sign = x.signum();
        }
    }
    let a: Vec<i64> = v
        .iter()
        .enumerate()
        .map(|(k, x)| {
            let x = x.abs();
            if x >= 9.223_372_036_854_775e18 {
                Err(elt_error(
                    i,
                    format!("duration field '{}' is out of range", DURATION_FIELDS[k]),
                ))
            } else {
                Ok(x as i64)
            }
        })
        .collect::<savvy::Result<_>>()?;
    let e = |e| elt_error(i, e);
    let span = Span::new()
        .try_years(a[0])
        .map_err(e)?
        .try_months(a[1])
        .map_err(e)?
        .try_weeks(a[2])
        .map_err(e)?
        .try_days(a[3])
        .map_err(e)?
        .try_hours(a[4])
        .map_err(e)?
        .try_minutes(a[5])
        .map_err(e)?
        .try_seconds(a[6])
        .map_err(e)?
        .try_milliseconds(a[7])
        .map_err(e)?
        .try_microseconds(a[8])
        .map_err(e)?
        .try_nanoseconds(a[9])
        .map_err(e)?;
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

pub(crate) struct DurationIn {
    cols: Vec<Vec<f64>>,
}

impl DurationIn {
    /// `x` is the list of the ten record fields, in `DURATION_FIELDS` order.
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        let mut cols = Vec::with_capacity(10);
        for (k, name) in DURATION_FIELDS.iter().enumerate() {
            let col = match x.get_by_index(k).map(|s| s.into_typed()) {
                Some(TypedSexp::Real(r)) => r,
                _ => {
                    return Err(savvy::Error::new(format!(
                        "internal error: duration field '{name}' must be a double vector"
                    )))
                }
            };
            cols.push(col.to_vec());
        }
        let lens: Vec<usize> = cols.iter().map(|c| c.len()).collect();
        common_len(&lens)?;
        Ok(Self { cols })
    }

    pub(crate) fn len(&self) -> usize {
        self.cols.first().map_or(0, |c| c.len())
    }

    pub(crate) fn get(&self, i: usize) -> savvy::Result<Option<Span>> {
        let mut v = [0.0; 10];
        for (k, c) in self.cols.iter().enumerate() {
            v[k] = c[i];
        }
        span_from_fields(i, v)
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
