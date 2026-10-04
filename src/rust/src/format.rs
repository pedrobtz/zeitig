//! Temporal `toString()` options, `strftime` and `strptime`.

use std::fmt::Write as _;

use jiff::civil::{Date, DateTime, Time};
use jiff::fmt::strtime::{self, BrokenDownTime};
use jiff::{RoundMode, Timestamp, Unit, Zoned};
use savvy::{savvy, ListSexp, OwnedStringSexp, StringSexp};

use crate::arith::{round_datetime, round_time};
use crate::cols::{
    common_len, elt_error, str_values, DateCols, DateOut, DateTimeCols, DateTimeOut, InstantCols,
    InstantOut, TimeCols, TimeOut, ZonedCols, ZonedOut,
};
use crate::opts::{increment_i64, parse_round_mode, parse_unit};
use crate::tz::{db, format_offset, time_zone_id, TzCache};
use crate::zoned::{instant_round_opts, round_zoned};

/// The record layout of a Temporal type, as named by the R callers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    PlainDate,
    PlainTime,
    PlainDateTime,
    Instant,
    Zoned,
}

impl Kind {
    fn parse(x: &str) -> savvy::Result<Self> {
        Ok(match x {
            "plain_date" => Kind::PlainDate,
            "plain_time" => Kind::PlainTime,
            "plain_date_time" => Kind::PlainDateTime,
            "instant" => Kind::Instant,
            "zoned_date_time" => Kind::Zoned,
            _ => {
                return Err(savvy::Error::new(format!(
                    "internal error: unknown kind '{x}'"
                )))
            }
        })
    }
}

/// Options of Temporal's `toString()`, already validated in R.
#[derive(Clone, Copy)]
pub(crate) struct FormatOpts {
    /// `None` = "auto" (shortest exact fraction), `Some(n)` = n digits.
    pub digits: Option<u8>,
    /// Print `HH:MM` only (`smallestUnit: "minute"`).
    pub minute: bool,
    pub offset: bool,
    /// 0 = never, 1 = auto, 2 = critical
    pub time_zone_name: u8,
    /// 0 = never/auto, 1 = always, 2 = critical
    pub calendar_name: u8,
}

impl Default for FormatOpts {
    fn default() -> Self {
        Self {
            digits: None,
            minute: false,
            offset: true,
            time_zone_name: 1,
            calendar_name: 0,
        }
    }
}

impl FormatOpts {
    fn from_r(
        digits: i32,
        minute: bool,
        offset: &str,
        time_zone_name: &str,
        calendar_name: &str,
    ) -> Self {
        Self {
            digits: if (0..=9).contains(&digits) {
                Some(digits as u8)
            } else {
                None
            },
            minute,
            offset: offset != "never",
            time_zone_name: match time_zone_name {
                "never" => 0,
                "critical" => 2,
                _ => 1,
            },
            calendar_name: match calendar_name {
                "always" => 1,
                "critical" => 2,
                _ => 0,
            },
        }
    }

    fn calendar(&self) -> &'static str {
        match self.calendar_name {
            1 => "[u-ca=iso8601]",
            2 => "[!u-ca=iso8601]",
            _ => "",
        }
    }
}

/// The rounding `toString()` applies before printing a fixed precision.
#[derive(Clone, Copy)]
pub(crate) struct Rounding {
    unit: Unit,
    increment: i64,
    mode: RoundMode,
}

// The writers below append to a buffer that is reused for every element, so
// formatting allocates nothing per element. Writing to a `String` cannot
// fail, hence the ignored `fmt::Result`s.

pub(crate) fn write_time(buf: &mut String, t: Time, o: &FormatOpts) {
    let _ = write!(buf, "{:02}:{:02}", t.hour(), t.minute());
    if o.minute {
        return;
    }
    let _ = write!(buf, ":{:02}", t.second());
    let ns = t.subsec_nanosecond();
    match o.digits {
        None => {
            if ns != 0 {
                let _ = write!(buf, ".{ns:09}");
                let trimmed = buf.trim_end_matches('0').len();
                buf.truncate(trimmed);
            }
        }
        Some(0) => {}
        Some(n) => {
            let start = buf.len();
            let _ = write!(buf, ".{ns:09}");
            buf.truncate(start + 1 + n as usize);
        }
    }
}

pub(crate) fn write_date(buf: &mut String, d: Date, o: &FormatOpts) {
    let _ = write!(buf, "{d}");
    buf.push_str(o.calendar());
}

pub(crate) fn write_datetime(buf: &mut String, dt: DateTime, o: &FormatOpts) {
    let _ = write!(buf, "{}T", dt.date());
    write_time(buf, dt.time(), o);
    buf.push_str(o.calendar());
}

pub(crate) fn write_zoned(buf: &mut String, z: &Zoned, id: &str, o: &FormatOpts) {
    let dt = z.datetime();
    let _ = write!(buf, "{}T", dt.date());
    write_time(buf, dt.time(), o);
    if o.offset {
        buf.push_str(&format_offset(z.offset().seconds()));
    }
    match o.time_zone_name {
        1 => {
            let _ = write!(buf, "[{id}]");
        }
        2 => {
            let _ = write!(buf, "[!{id}]");
        }
        _ => {}
    }
    buf.push_str(o.calendar());
}

/// Temporal `Instant.prototype.toString()`: UTC with `Z`, or the wall clock
/// and offset in `tz` when one is given (no annotation).
pub(crate) fn write_instant(buf: &mut String, t: Timestamp, tz: Option<&Zoned>, o: &FormatOpts) {
    match tz {
        None => {
            let dt = t.to_zoned(jiff::tz::TimeZone::UTC).datetime();
            let _ = write!(buf, "{}T", dt.date());
            write_time(buf, dt.time(), o);
            buf.push('Z');
        }
        Some(z) => {
            let dt = z.datetime();
            let _ = write!(buf, "{}T", dt.date());
            write_time(buf, dt.time(), o);
            buf.push_str(&format_offset(z.offset().seconds()));
        }
    }
}

#[cfg(test)]
fn fmt_with(f: impl FnOnce(&mut String)) -> String {
    let mut s = String::new();
    f(&mut s);
    s
}

/// Formats every element of a record with `o`, after `round` when given.
/// `time_zone` (instants only, same length as `x`) prints the wall clock in
/// that zone.
fn format_records(
    x: &ListSexp,
    kind: Kind,
    o: &FormatOpts,
    round: Option<Rounding>,
    time_zone: Option<&StringSexp>,
) -> savvy::Result<savvy::Sexp> {
    let mut buf = String::with_capacity(64);
    macro_rules! each {
        ($n:expr, |$i:ident, $buf:ident| $body:block) => {{
            let n = $n;
            let mut out = OwnedStringSexp::new(n)?;
            for $i in 0..n {
                buf.clear();
                let $buf = &mut buf;
                let written: bool = $body;
                if written {
                    out.set_elt($i, $buf)?;
                } else {
                    out.set_na($i)?;
                }
            }
            Ok(out.into())
        }};
    }
    let rerr = |i: usize| move |e: jiff::Error| elt_error(i, e);
    match kind {
        Kind::PlainDate => {
            let cols = DateCols::new(x)?;
            let r = cols.reader()?;
            each!(r.len(), |i, b| {
                match r.get(i)? {
                    Some(d) => {
                        write_date(b, d, o);
                        true
                    }
                    None => false,
                }
            })
        }
        Kind::PlainTime => {
            let cols = TimeCols::new(x)?;
            let r = cols.reader()?;
            each!(r.len(), |i, b| {
                match r.get(i)? {
                    Some(t) => {
                        let t = match round {
                            Some(p) => {
                                round_time(t, p.unit, p.increment, p.mode).map_err(rerr(i))?
                            }
                            None => t,
                        };
                        write_time(b, t, o);
                        b.push_str(o.calendar());
                        true
                    }
                    None => false,
                }
            })
        }
        Kind::PlainDateTime => {
            let cols = DateTimeCols::new(x)?;
            let r = cols.reader()?;
            each!(r.len(), |i, b| {
                match r.get(i)? {
                    Some(dt) => {
                        let dt = match round {
                            Some(p) => {
                                round_datetime(dt, p.unit, p.increment, p.mode).map_err(rerr(i))?
                            }
                            None => dt,
                        };
                        write_datetime(b, dt, o);
                        true
                    }
                    None => false,
                }
            })
        }
        Kind::Instant => {
            let cols = InstantCols::new(x)?;
            let r = cols.reader()?;
            let tz = time_zone.map(str_values);
            if let Some(tz) = &tz {
                common_len(&[r.len(), tz.len()])?;
            }
            let opts = round.map(|p| instant_round_opts(p.unit, p.increment, p.mode));
            let mut cache = TzCache::default();
            each!(r.len(), |i, b| {
                match r.get(i)? {
                    Some(t) => {
                        let t = match opts {
                            Some(opts) => t.round(opts).map_err(rerr(i))?,
                            None => t,
                        };
                        match tz.as_ref().and_then(|tz| tz[i]) {
                            Some(id) => {
                                let zone = cache.get(i, id)?.tz.clone();
                                write_instant(b, t, Some(&t.to_zoned(zone)), o);
                            }
                            None => write_instant(b, t, None, o),
                        }
                        true
                    }
                    None => false,
                }
            })
        }
        Kind::Zoned => {
            let cols = ZonedCols::new(x)?;
            let r = cols.reader()?;
            let mut cache = TzCache::default();
            each!(r.len(), |i, b| {
                match r.get(i, &mut cache)? {
                    Some((z, id)) => {
                        let z = match round {
                            Some(p) => {
                                round_zoned(&z, p.unit, p.increment, p.mode).map_err(rerr(i))?
                            }
                            None => z,
                        };
                        write_zoned(b, &z, &id, o);
                        true
                    }
                    None => false,
                }
            })
        }
    }
}

/// `toString()` with default options.
pub(crate) fn format_default(x: &ListSexp, kind: Kind) -> savvy::Result<savvy::Sexp> {
    format_records(x, kind, &FormatOpts::default(), None, None)
}

// Formats any Temporal record with `toString()` options. `kind` names the
// record layout: "plain_date", "plain_time", "plain_date_time", "instant",
// "zoned_date_time". When `round_unit` is not empty, each value is first
// rounded to `round_increment` `round_unit`s with `round_mode` (Temporal
// rounds before printing a fixed precision). `time_zone` (instants only) is
// either NULL or a character vector as long as `x`, NA for UTC.
#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_format(
    x: ListSexp,
    kind: &str,
    digits: i32,
    minute: bool,
    offset: &str,
    time_zone_name: &str,
    calendar_name: &str,
    round_unit: &str,
    round_increment: f64,
    round_mode: &str,
    time_zone: Option<StringSexp>,
) -> savvy::Result<savvy::Sexp> {
    let kind = Kind::parse(kind)?;
    let o = FormatOpts::from_r(digits, minute, offset, time_zone_name, calendar_name);
    let round = if round_unit.is_empty() || kind == Kind::PlainDate {
        None
    } else {
        Some(Rounding {
            unit: parse_unit(round_unit)?,
            increment: increment_i64(round_increment)?,
            mode: parse_round_mode(round_mode)?,
        })
    };
    format_records(&x, kind, &o, round, time_zone.as_ref())
}

// `format` must be as long as `x` (R recycles); NA formats give NA.
#[savvy]
fn rs_strftime(x: ListSexp, kind: &str, format: StringSexp) -> savvy::Result<savvy::Sexp> {
    let fmts = str_values(&format);
    let mut buf = String::with_capacity(64);
    let mut write =
        |out: &mut OwnedStringSexp, i: usize, tm: Option<BrokenDownTime>| -> savvy::Result<()> {
            match (tm, fmts[i]) {
                (Some(tm), Some(f)) => {
                    buf.clear();
                    tm.format(f, &mut buf).map_err(|e| elt_error(i, e))?;
                    out.set_elt(i, &buf)?;
                }
                _ => out.set_na(i)?,
            }
            Ok(())
        };
    macro_rules! each {
        ($len:expr, |$i:ident| $get:expr) => {{
            let n = common_len(&[$len, fmts.len()])?;
            let mut out = OwnedStringSexp::new(n)?;
            for $i in 0..n {
                let tm = $get.map(BrokenDownTime::from);
                write(&mut out, $i, tm)?;
            }
            Ok(out.into())
        }};
    }
    match Kind::parse(kind)? {
        Kind::PlainDate => {
            let cols = DateCols::new(&x)?;
            let r = cols.reader()?;
            each!(r.len(), |i| r.get(i)?)
        }
        Kind::PlainTime => {
            let cols = TimeCols::new(&x)?;
            let r = cols.reader()?;
            each!(r.len(), |i| r.get(i)?)
        }
        Kind::PlainDateTime => {
            let cols = DateTimeCols::new(&x)?;
            let r = cols.reader()?;
            each!(r.len(), |i| r.get(i)?)
        }
        Kind::Instant => {
            let cols = InstantCols::new(&x)?;
            let r = cols.reader()?;
            each!(r.len(), |i| r.get(i)?)
        }
        Kind::Zoned => {
            let cols = ZonedCols::new(&x)?;
            let r = cols.reader()?;
            let mut cache = TzCache::default();
            each!(r.len(), |i| r.get(i, &mut cache)?.map(|(z, _)| z).as_ref())
        }
    }
}

// `format` must be as long as `x` (R recycles).
#[savvy]
fn rs_strptime(x: StringSexp, format: StringSexp, kind: &str) -> savvy::Result<savvy::Sexp> {
    let input = str_values(&x);
    let fmts = str_values(&format);
    let n = common_len(&[input.len(), fmts.len()])?;
    let parsed = |i: usize| -> savvy::Result<Option<BrokenDownTime>> {
        match (input[i], fmts[i]) {
            (Some(s), Some(f)) => strtime::parse(f, s).map(Some).map_err(|e| elt_error(i, e)),
            _ => Ok(None),
        }
    };
    match Kind::parse(kind)? {
        Kind::PlainDate => {
            let mut out = DateOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_date().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        Kind::PlainTime => {
            let mut out = TimeOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_time().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        Kind::PlainDateTime => {
            let mut out = DateTimeOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_datetime().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        Kind::Instant => {
            let mut out = InstantOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_timestamp().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        Kind::Zoned => {
            let db = db().map_err(savvy::Error::new)?;
            let mut cache = TzCache::default();
            let mut out = ZonedOut::with_capacity(n);
            for i in 0..n {
                let Some(b) = parsed(i)? else {
                    out.push(None);
                    continue;
                };
                let z = b.to_zoned_with(db).map_err(|e| elt_error(i, e))?;
                // Canonical identifier, as for parsed RFC 9557 strings.
                let id = time_zone_id(z.time_zone()).map_err(|e| elt_error(i, e))?;
                let r = cache.get(i, &id)?;
                let z = if r.tz == *z.time_zone() {
                    z
                } else {
                    z.with_time_zone(r.tz.clone())
                };
                out.push(Some((&z, &r.id)));
            }
            out.into_sexp()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::{date, time};
    use jiff::tz::TimeZone;

    /// Default options must reproduce jiff's `Display` (Temporal format).
    #[test]
    fn defaults_match_display() {
        let o = FormatOpts::default();
        for ns in [0, 1, 500_000_000, 123_456_789, 100] {
            let t = time(1, 2, 3, ns);
            assert_eq!(fmt_with(|b| write_time(b, t, &o)), t.to_string());
            let dt = date(2020, 2, 29).to_datetime(t);
            assert_eq!(fmt_with(|b| write_datetime(b, dt, &o)), dt.to_string());
            for tz in ["America/New_York", "UTC", "Asia/Kolkata"] {
                let z = dt.to_zoned(TimeZone::get(tz).unwrap()).unwrap();
                assert_eq!(fmt_with(|b| write_zoned(b, &z, tz, &o)), z.to_string());
            }
            let fixed = dt.to_zoned(TimeZone::fixed(jiff::tz::offset(-8))).unwrap();
            assert_eq!(
                fmt_with(|b| write_zoned(b, &fixed, "-08:00", &o)),
                fixed.to_string()
            );
            let ts = dt.to_zoned(TimeZone::UTC).unwrap().timestamp();
            assert_eq!(fmt_with(|b| write_instant(b, ts, None, &o)), ts.to_string());
        }
        let d = date(-5, 1, 1);
        assert_eq!(fmt_with(|b| write_date(b, d, &o)), d.to_string());
    }

    #[test]
    fn options() {
        let t = time(15, 23, 30, 120_000_000);
        let o = |digits, minute| FormatOpts {
            digits,
            minute,
            ..FormatOpts::default()
        };
        let ft = |o: FormatOpts| fmt_with(|b| write_time(b, t, &o));
        assert_eq!(ft(o(Some(0), false)), "15:23:30");
        assert_eq!(ft(o(Some(4), false)), "15:23:30.1200");
        assert_eq!(ft(o(Some(9), false)), "15:23:30.120000000");
        assert_eq!(ft(o(None, true)), "15:23");
        // the buffer is reused: trimming must not eat earlier zeros
        let mut b = String::from("2020-10-10T");
        write_time(&mut b, time(10, 0, 0, 500_000_000), &o(None, false));
        assert_eq!(b, "2020-10-10T10:00:00.5");
        let mut b = String::from("2020-10-10T");
        write_time(&mut b, time(10, 0, 0, 0), &o(None, false));
        assert_eq!(b, "2020-10-10T10:00:00");
        let z = date(2020, 1, 1)
            .to_datetime(t)
            .to_zoned(TimeZone::get("Europe/Paris").unwrap())
            .unwrap();
        let mut opts = FormatOpts {
            offset: false,
            time_zone_name: 2,
            calendar_name: 1,
            ..FormatOpts::default()
        };
        assert_eq!(
            fmt_with(|b| write_zoned(b, &z, "Europe/Paris", &opts)),
            "2020-01-01T15:23:30.12[!Europe/Paris][u-ca=iso8601]"
        );
        opts.time_zone_name = 0;
        opts.calendar_name = 0;
        opts.offset = true;
        assert_eq!(
            fmt_with(|b| write_zoned(b, &z, "Europe/Paris", &opts)),
            "2020-01-01T15:23:30.12+01:00"
        );
    }
}
