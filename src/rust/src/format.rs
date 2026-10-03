//! Temporal `toString()` options, `strftime` and `strptime`.

use jiff::civil::{Date, DateTime, Time};
use jiff::fmt::strtime;
use jiff::{Timestamp, Zoned};
use savvy::{savvy, ListSexp, OwnedStringSexp, StringSexp};

use crate::cols::{
    elt_error, is_na_str, list_int, DateIn, DateOut, DateTimeOut, InstantIn, InstantOut, TimeIn,
    TimeOut, ZonedIn, ZonedOut,
};
use crate::duration::DateTimeCols;
use crate::tz::{format_offset, TzCache};

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
    ) -> savvy::Result<Self> {
        Ok(Self {
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
        })
    }

    fn calendar(&self) -> &'static str {
        match self.calendar_name {
            1 => "[u-ca=iso8601]",
            2 => "[!u-ca=iso8601]",
            _ => "",
        }
    }
}

pub(crate) fn fmt_time(t: Time, o: &FormatOpts) -> String {
    let mut s = format!("{:02}:{:02}", t.hour(), t.minute());
    if o.minute {
        return s;
    }
    s.push_str(&format!(":{:02}", t.second()));
    let ns = t.subsec_nanosecond();
    match o.digits {
        None => {
            if ns != 0 {
                let frac = format!("{ns:09}");
                s.push('.');
                s.push_str(frac.trim_end_matches('0'));
            }
        }
        Some(0) => {}
        Some(n) => {
            let frac = format!("{ns:09}");
            s.push('.');
            s.push_str(&frac[..n as usize]);
        }
    }
    s
}

pub(crate) fn fmt_date(d: Date, o: &FormatOpts) -> String {
    format!("{d}{}", o.calendar())
}

pub(crate) fn fmt_datetime(dt: DateTime, o: &FormatOpts) -> String {
    format!("{}T{}{}", dt.date(), fmt_time(dt.time(), o), o.calendar())
}

pub(crate) fn fmt_zoned(z: &Zoned, id: &str, o: &FormatOpts) -> String {
    let dt = z.datetime();
    let mut s = format!("{}T{}", dt.date(), fmt_time(dt.time(), o));
    if o.offset {
        s.push_str(&format_offset(z.offset().seconds()));
    }
    match o.time_zone_name {
        1 => s.push_str(&format!("[{id}]")),
        2 => s.push_str(&format!("[!{id}]")),
        _ => {}
    }
    s.push_str(o.calendar());
    s
}

/// Temporal `Instant.prototype.toString()`: UTC with `Z`, or the wall clock
/// and offset in `tz` when one is given (no annotation).
pub(crate) fn fmt_instant(t: Timestamp, tz: Option<&Zoned>, o: &FormatOpts) -> String {
    match tz {
        None => {
            let dt = t.to_zoned(jiff::tz::TimeZone::UTC).datetime();
            format!("{}T{}Z", dt.date(), fmt_time(dt.time(), o))
        }
        Some(z) => {
            let dt = z.datetime();
            format!(
                "{}T{}{}",
                dt.date(),
                fmt_time(dt.time(), o),
                format_offset(z.offset().seconds())
            )
        }
    }
}

fn strings(v: Vec<Option<String>>) -> savvy::Result<savvy::Sexp> {
    let mut out = OwnedStringSexp::new(v.len())?;
    for (i, s) in v.iter().enumerate() {
        match s {
            Some(s) => out.set_elt(i, s)?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// Formats any Temporal record with `toString()` options. `kind` names the
// record layout: "plain_date", "plain_time", "plain_date_time", "instant",
// "zoned_date_time". `time_zone` (instants only) may be NA per element.
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
    time_zone: StringSexp,
) -> savvy::Result<savvy::Sexp> {
    let o = FormatOpts::from_r(digits, minute, offset, time_zone_name, calendar_name)?;
    let mut out = Vec::new();
    match kind {
        "plain_date" => {
            let (y, m, d) = (int_sexp(&x, 0)?, int_sexp(&x, 1)?, int_sexp(&x, 2)?);
            let r = DateIn::new(&y, &m, &d)?;
            for i in 0..r.len() {
                out.push(r.get(i)?.map(|v| fmt_date(v, &o)));
            }
        }
        "plain_time" => {
            let (s, n) = (int_sexp(&x, 0)?, int_sexp(&x, 1)?);
            let r = TimeIn::new(&s, &n)?;
            for i in 0..r.len() {
                out.push(
                    r.get(i)?
                        .map(|v| format!("{}{}", fmt_time(v, &o), o.calendar())),
                );
            }
        }
        "plain_date_time" => {
            let cols = DateTimeCols::new(&x)?;
            let r = cols.reader()?;
            for i in 0..r.len() {
                out.push(r.get(i)?.map(|v| fmt_datetime(v, &o)));
            }
        }
        "instant" => {
            let r = InstantIn::new(&x)?;
            let tz: Vec<&str> = time_zone.iter().collect();
            let mut cache = TzCache::default();
            for i in 0..r.len() {
                let Some(t) = r.get(i)? else {
                    out.push(None);
                    continue;
                };
                let id = tz
                    .get(i % tz.len().max(1))
                    .copied()
                    .filter(|s| !is_na_str(s));
                match id {
                    Some(id) => {
                        let zone = cache.get(i, id)?.0.clone();
                        out.push(Some(fmt_instant(t, Some(&t.to_zoned(zone)), &o)));
                    }
                    None => out.push(Some(fmt_instant(t, None, &o))),
                }
            }
        }
        "zoned_date_time" => {
            let r = ZonedIn::new(&x)?;
            let mut cache = TzCache::default();
            for i in 0..r.len() {
                match r.get(i, &mut cache)? {
                    Some(z) => {
                        let id = crate::tz::time_zone_id(z.time_zone());
                        out.push(Some(fmt_zoned(&z, &id, &o)));
                    }
                    None => out.push(None),
                }
            }
        }
        _ => {
            return Err(savvy::Error::new(format!(
                "internal error: unknown kind '{kind}'"
            )))
        }
    }
    strings(out)
}

fn int_sexp(x: &ListSexp, k: usize) -> savvy::Result<savvy::IntegerSexp> {
    let v = list_int(x, k)?;
    Ok(savvy::OwnedIntegerSexp::try_from_slice(v)?.as_read_only())
}

#[savvy]
fn rs_strftime(x: ListSexp, kind: &str, format: StringSexp) -> savvy::Result<savvy::Sexp> {
    let fmts: Vec<&str> = format.iter().collect();
    if fmts.is_empty() {
        return Err(savvy::Error::new("`format` must not be empty"));
    }
    let fmt_at = |i: usize| fmts[i % fmts.len()];
    let mut out = Vec::new();
    macro_rules! each {
        ($len:expr, $get:expr) => {
            for i in 0..$len {
                let f = fmt_at(i);
                match $get(i)? {
                    Some(v) if !is_na_str(f) => {
                        out.push(Some(strtime::format(f, v).map_err(|e| elt_error(i, e))?))
                    }
                    _ => out.push(None),
                }
            }
        };
    }
    match kind {
        "plain_date" => {
            let (y, m, d) = (int_sexp(&x, 0)?, int_sexp(&x, 1)?, int_sexp(&x, 2)?);
            let r = DateIn::new(&y, &m, &d)?;
            each!(r.len(), |i| r.get(i));
        }
        "plain_time" => {
            let (s, n) = (int_sexp(&x, 0)?, int_sexp(&x, 1)?);
            let r = TimeIn::new(&s, &n)?;
            each!(r.len(), |i| r.get(i));
        }
        "plain_date_time" => {
            let cols = DateTimeCols::new(&x)?;
            let r = cols.reader()?;
            each!(r.len(), |i| r.get(i));
        }
        "instant" => {
            let r = InstantIn::new(&x)?;
            each!(r.len(), |i| r.get(i));
        }
        "zoned_date_time" => {
            let r = ZonedIn::new(&x)?;
            let mut cache = TzCache::default();
            for i in 0..r.len() {
                let f = fmt_at(i);
                match r.get(i, &mut cache)? {
                    Some(z) if !is_na_str(f) => {
                        out.push(Some(strtime::format(f, &z).map_err(|e| elt_error(i, e))?))
                    }
                    _ => out.push(None),
                }
            }
        }
        _ => {
            return Err(savvy::Error::new(format!(
                "internal error: unknown kind '{kind}'"
            )))
        }
    }
    strings(out)
}

#[savvy]
fn rs_strptime(x: StringSexp, format: StringSexp, kind: &str) -> savvy::Result<savvy::Sexp> {
    let fmts: Vec<&str> = format.iter().collect();
    if fmts.is_empty() {
        return Err(savvy::Error::new("`format` must not be empty"));
    }
    let n = x.len();
    let input: Vec<&str> = x.iter().collect();
    let parsed = |i: usize| -> savvy::Result<Option<strtime::BrokenDownTime>> {
        let (s, f) = (input[i], fmts[i % fmts.len()]);
        if is_na_str(s) || is_na_str(f) {
            return Ok(None);
        }
        strtime::parse(f, s).map(Some).map_err(|e| elt_error(i, e))
    };
    match kind {
        "plain_date" => {
            let mut out = DateOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_date().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        "plain_time" => {
            let mut out = TimeOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_time().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        "plain_date_time" => {
            let mut out = DateTimeOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_datetime().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        "instant" => {
            let mut out = InstantOut::with_capacity(n);
            for i in 0..n {
                out.push(match parsed(i)? {
                    Some(b) => Some(b.to_timestamp().map_err(|e| elt_error(i, e))?),
                    None => None,
                });
            }
            out.into_sexp()
        }
        "zoned_date_time" => {
            let mut out = ZonedOut::with_capacity(n);
            for i in 0..n {
                let z = match parsed(i)? {
                    Some(b) => Some(b.to_zoned().map_err(|e| elt_error(i, e))?),
                    None => None,
                };
                out.push(z.as_ref());
            }
            out.into_sexp()
        }
        _ => Err(savvy::Error::new(format!(
            "internal error: unknown kind '{kind}'"
        ))),
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
            assert_eq!(fmt_time(t, &o), t.to_string());
            let dt = date(2020, 2, 29).to_datetime(t);
            assert_eq!(fmt_datetime(dt, &o), dt.to_string());
            for tz in ["America/New_York", "UTC", "Asia/Kolkata"] {
                let z = dt.to_zoned(TimeZone::get(tz).unwrap()).unwrap();
                assert_eq!(fmt_zoned(&z, tz, &o), z.to_string());
            }
            let fixed = dt.to_zoned(TimeZone::fixed(jiff::tz::offset(-8))).unwrap();
            assert_eq!(fmt_zoned(&fixed, "-08:00", &o), fixed.to_string());
            let ts = dt.to_zoned(TimeZone::UTC).unwrap().timestamp();
            assert_eq!(fmt_instant(ts, None, &o), ts.to_string());
        }
        assert_eq!(fmt_date(date(-5, 1, 1), &o), date(-5, 1, 1).to_string());
    }

    #[test]
    fn options() {
        let t = time(15, 23, 30, 120_000_000);
        let o = |digits, minute| FormatOpts {
            digits,
            minute,
            ..FormatOpts::default()
        };
        assert_eq!(fmt_time(t, &o(Some(0), false)), "15:23:30");
        assert_eq!(fmt_time(t, &o(Some(4), false)), "15:23:30.1200");
        assert_eq!(fmt_time(t, &o(None, true)), "15:23");
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
            fmt_zoned(&z, "Europe/Paris", &opts),
            "2020-01-01T15:23:30.12[!Europe/Paris][u-ca=iso8601]"
        );
        opts.time_zone_name = 0;
        opts.calendar_name = 0;
        opts.offset = true;
        assert_eq!(
            fmt_zoned(&z, "Europe/Paris", &opts),
            "2020-01-01T15:23:30.12+01:00"
        );
    }
}
