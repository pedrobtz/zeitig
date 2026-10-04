//! Temporal `Instant` (jiff `Timestamp`), `ZonedDateTime` (jiff `Zoned`) and
//! `Now`.

use std::rc::Rc;

use jiff::civil::DateTime;
use jiff::fmt::temporal::DateTimeParser;
use jiff::tz::{Disambiguation, Offset, OffsetConflict};
use jiff::{RoundMode, Timestamp, TimestampRound, Unit, Zoned, ZonedRound};
use savvy::{savvy, ListSexp, OwnedIntegerSexp, OwnedRealSexp, OwnedStringSexp, StringSexp};

use crate::arith::{check_reject, difference, round_time_like};
use crate::cols::{
    common_len, elt_error, is_na_str, str_values, DateTimeCols, DateTimeOut, DurationCols,
    DurationOut, InstantCols, InstantOut, ZonedCols, ZonedOut,
};
use crate::ixdtf::{prepare, Kind};
use crate::opts::{increment_i64, parse_round_mode, parse_unit, DiffOpts};
use crate::tz::{
    db, format_offset, parse_disambiguation, parse_offset_conflict, time_zone_id, time_zones_equal,
    TzCache,
};

fn parser() -> DateTimeParser {
    DateTimeParser::new()
}

// ---------------------------------------------------------------------------
// Instant

#[savvy]
fn rs_instant_parse(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let p = parser();
    let mut out = InstantOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if is_na_str(s) {
            out.push(None);
        } else {
            let s = prepare(s, Kind::Instant).map_err(|e| elt_error(i, e))?;
            out.push(Some(p.parse_timestamp(&*s).map_err(|e| elt_error(i, e))?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_instant_validate(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = InstantCols::new(&x)?;
    let x = cols.reader()?;
    let mut out = InstantOut::with_capacity(x.len());
    for i in 0..x.len() {
        out.push(x.get(i)?);
    }
    out.into_sexp()
}

#[savvy]
fn rs_instant_format(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = InstantCols::new(&x)?;
    let x = cols.reader()?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.set_elt(i, &t.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// Instants from epoch nanoseconds given as decimal strings (R has no 64-bit
// integers).
#[savvy]
fn rs_instant_from_epoch_nanoseconds(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = InstantOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if is_na_str(s) {
            out.push(None);
            continue;
        }
        let ns: i128 = s
            .trim()
            .parse()
            .map_err(|_| elt_error(i, format!("'{s}' is not an integer number of nanoseconds")))?;
        // Checked here because jiff panics for some values far out of range.
        let (lo, hi) = (
            Timestamp::MIN.as_nanosecond(),
            Timestamp::MAX.as_nanosecond(),
        );
        if !(lo..=hi).contains(&ns) {
            return Err(elt_error(
                i,
                format!("{s} nanoseconds since the epoch is outside the supported range"),
            ));
        }
        out.push(Some(
            Timestamp::from_nanosecond(ns).map_err(|e| elt_error(i, e))?,
        ));
    }
    out.into_sexp()
}

#[savvy]
fn rs_instant_epoch_nanoseconds(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = InstantCols::new(&x)?;
    let x = cols.reader()?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.set_elt(i, &t.as_nanosecond().to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

#[savvy]
fn rs_instant_add(x: ListSexp, duration: ListSexp) -> savvy::Result<savvy::Sexp> {
    let (xc, dc) = (InstantCols::new(&x)?, DurationCols::new(&duration)?);
    let (x, d) = (xc.reader()?, dc.reader()?);
    let n = common_len(&[x.len(), d.len()])?;
    let mut out = InstantOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, d.get(i)?) {
            (Some(t), Some(span)) => {
                out.push(Some(t.checked_add(span).map_err(|e| elt_error(i, e))?))
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_instant_diff(
    x: ListSexp,
    y: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    since: bool,
) -> savvy::Result<savvy::Sexp> {
    let opts = DiffOpts::new(largest, smallest, increment, mode)?;
    let (xc, yc) = (InstantCols::new(&x)?, InstantCols::new(&y)?);
    let (a, b) = (xc.reader()?, yc.reader()?);
    let n = common_len(&[a.len(), b.len()])?;
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i)?, b.get(i)?) {
            (Some(a), Some(b)) => {
                let r = difference(&a, &b, &opts, since);
                out.push(Some(r.map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_instant_round(
    x: ListSexp,
    smallest: &str,
    increment: f64,
    mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let cols = InstantCols::new(&x)?;
    let x = cols.reader()?;
    let opts = instant_round_opts(
        parse_unit(smallest)?,
        increment_i64(increment)?,
        parse_round_mode(mode)?,
    );
    let mut out = InstantOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.push(Some(t.round(opts).map_err(|e| elt_error(i, e))?)),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

/// Options for `Instant.prototype.round()`. Temporal rounds instants "as if
/// positive" (RoundNumberToIncrementAsIfPositive): `trunc` goes down even
/// before 1970, while jiff rounds the signed epoch value towards zero.
/// Mapping each mode to its direction-fixed counterpart gives Temporal's
/// result for every instant.
pub(crate) fn instant_round_opts(unit: Unit, increment: i64, mode: RoundMode) -> TimestampRound {
    let mode = match mode {
        RoundMode::Trunc => RoundMode::Floor,
        RoundMode::Expand => RoundMode::Ceil,
        RoundMode::HalfTrunc => RoundMode::HalfFloor,
        RoundMode::HalfExpand => RoundMode::HalfCeil,
        m => m,
    };
    TimestampRound::new()
        .smallest(unit)
        .increment(increment)
        .mode(mode)
}

/// `ZonedDateTime.prototype.round()` for one value.
pub(crate) fn round_zoned(
    z: &Zoned,
    unit: Unit,
    increment: i64,
    mode: RoundMode,
) -> Result<Zoned, jiff::Error> {
    let opts = ZonedRound::new().smallest(unit).increment(increment);
    round_time_like(
        unit,
        increment,
        mode,
        |m| z.round(opts.mode(m)),
        |z| z.time(),
    )
}

#[savvy]
fn rs_now() -> savvy::Result<savvy::Sexp> {
    let mut out = InstantOut::with_capacity(1);
    out.push(Some(Timestamp::now()));
    out.into_sexp()
}

// ---------------------------------------------------------------------------
// ZonedDateTime

// Instants (seconds, nanos) viewed in time zones -> zoned date-times. Also
// used by `with_time_zone()` and to validate/canonicalise identifiers.
#[savvy]
fn rs_instant_to_zoned(x: ListSexp, time_zone: StringSexp) -> savvy::Result<savvy::Sexp> {
    let cols = InstantCols::new(&x)?;
    let x = cols.reader()?;
    let tz = str_values(&time_zone);
    let n = common_len(&[x.len(), tz.len()])?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for (i, id) in tz.iter().enumerate() {
        match (x.get(i)?, id) {
            (Some(t), Some(id)) => {
                let r = cache.get(i, id)?;
                out.push(Some((&t.to_zoned(r.tz.clone()), &r.id)));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

// Plain date-times in time zones -> zoned date-times, resolving gaps and
// overlaps with `disambiguation`. When `reference` (zoned date-times, as in
// `ZonedDateTime.prototype.with()`) is given, each element's UTC offset is
// reconciled with the new wall-clock time using `offset_mode` (Temporal's
// `offset` option).
#[savvy]
fn rs_zoned_from_civil(
    x: ListSexp,
    time_zone: StringSexp,
    disambiguation: &str,
    offset_mode: &str,
    reference: Option<ListSexp>,
) -> savvy::Result<savvy::Sexp> {
    let cols = DateTimeCols::new(&x)?;
    let x = cols.reader()?;
    let tz = str_values(&time_zone);
    let n = common_len(&[x.len(), tz.len()])?;
    let ref_cols = reference.as_ref().map(ZonedCols::new).transpose()?;
    let reference = ref_cols.as_ref().map(|c| c.reader()).transpose()?;
    if let Some(r) = &reference {
        common_len(&[n, r.len()])?;
    }
    let disambiguation = parse_disambiguation(disambiguation)?;
    let conflict = parse_offset_conflict(offset_mode)?;
    let mut cache = TzCache::default();
    let mut ref_cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for (i, id) in tz.iter().enumerate() {
        let (Some(dt), Some(id)) = (x.get(i)?, id) else {
            out.push(None);
            continue;
        };
        let offset = match &reference {
            Some(r) => r.get(i, &mut ref_cache)?.map(|(z, _)| z.offset()),
            None => None,
        };
        let r = cache.get(i, id)?;
        let z = zoned_from_civil(i, dt, r.tz.clone(), offset, conflict, disambiguation)?;
        out.push(Some((&z, &r.id)));
    }
    out.into_sexp()
}

fn zoned_from_civil(
    i: usize,
    dt: DateTime,
    zone: jiff::tz::TimeZone,
    offset: Option<Offset>,
    conflict: OffsetConflict,
    disambiguation: Disambiguation,
) -> savvy::Result<Zoned> {
    let ambiguous = match offset {
        None => zone.to_ambiguous_zoned(dt),
        Some(off) => conflict
            .resolve(dt, off, zone)
            .map_err(|e| elt_error(i, e))?,
    };
    ambiguous
        .disambiguate(disambiguation)
        .map_err(|e| elt_error(i, e))
}

#[savvy]
fn rs_zoned_parse(
    x: StringSexp,
    disambiguation: &str,
    offset_mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let p = parser()
        .disambiguation(parse_disambiguation(disambiguation)?)
        .offset_conflict(parse_offset_conflict(offset_mode)?);
    let db = db().map_err(savvy::Error::new)?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if is_na_str(s) {
            out.push(None);
            continue;
        }
        let s = prepare(s, Kind::Zoned).map_err(|e| elt_error(i, e))?;
        let z = p.parse_zoned_with(db, &*s).map_err(|e| elt_error(i, e))?;
        // The zone is re-resolved through the cache so names and fixed
        // offsets are canonical, POSIX TZ strings (not Temporal identifiers)
        // are rejected, and a `[+00:00]` annotation stays `+00:00` (jiff
        // turns it into UTC, which Temporal keeps apart).
        let r = match offset_annotation(&s) {
            Some(ann) => cache.get(i, ann)?,
            None => match z.time_zone().iana_name() {
                Some(name) => cache.get(i, name)?,
                None => {
                    let id = time_zone_id(z.time_zone()).map_err(|e| elt_error(i, e))?;
                    cache.get(i, &id)?
                }
            },
        };
        let z = if r.tz == *z.time_zone() {
            z
        } else {
            z.with_time_zone(r.tz.clone())
        };
        out.push(Some((&z, &r.id)));
    }
    out.into_sexp()
}

/// The time zone annotation of a zoned string when it is a UTC offset.
fn offset_annotation(s: &str) -> Option<&str> {
    let start = s.find('[')? + 1;
    let ann = &s[start..start + s[start..].find(']')?];
    let ann = ann.strip_prefix('!').unwrap_or(ann);
    ann.starts_with(['+', '-']).then_some(ann)
}

#[savvy]
fn rs_zoned_format(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    crate::format::format_default(&x, crate::format::Kind::Zoned)
}

// The wall-clock date-time of each element.
#[savvy]
fn rs_zoned_civil(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let mut cache = TzCache::default();
    let mut out = DateTimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        out.push(x.get(i, &mut cache)?.map(|(z, _)| z.datetime()));
    }
    out.into_sexp()
}

// The UTC offset of each element: a `+HH:MM` string when `as_string`, else
// an integer number of seconds.
#[savvy]
fn rs_zoned_offset(x: ListSexp, as_string: bool) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let mut cache = TzCache::default();
    let n = x.len();
    if as_string {
        let mut out = OwnedStringSexp::new(n)?;
        for i in 0..n {
            match x.get(i, &mut cache)? {
                Some((z, _)) => out.set_elt(i, &format_offset(z.offset().seconds()))?,
                None => out.set_na(i)?,
            }
        }
        Ok(out.into())
    } else {
        let mut out = OwnedIntegerSexp::new(n)?;
        for i in 0..n {
            match x.get(i, &mut cache)? {
                Some((z, _)) => out.set_elt(i, z.offset().seconds())?,
                None => out.set_na(i)?,
            }
        }
        Ok(out.into())
    }
}

#[savvy]
fn rs_zoned_hours_in_day(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let mut cache = TzCache::default();
    let mut out = OwnedRealSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some((z, _)) => {
                let start = z.start_of_day().map_err(|e| elt_error(i, e))?;
                let next = z
                    .date()
                    .tomorrow()
                    .and_then(|d| d.to_zoned(z.time_zone().clone()))
                    .and_then(|d| d.start_of_day())
                    .map_err(|e| elt_error(i, e))?;
                let secs = next.timestamp().as_second() - start.timestamp().as_second();
                out.set_elt(i, secs as f64 / 3600.0)?;
            }
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

#[savvy]
fn rs_zoned_start_of_day(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some((z, id)) => {
                let start = z.start_of_day().map_err(|e| elt_error(i, e))?;
                out.push(Some((&start, &id)));
            }
            None => out.push(None),
        }
    }
    out.into_sexp()
}

// The next or previous UTC offset transition, `NA` when there is none.
#[savvy]
fn rs_zoned_transition(x: ListSexp, next: bool) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        let Some((z, id)) = x.get(i, &mut cache)? else {
            out.push(None);
            continue;
        };
        let tz = z.time_zone();
        let t = z.timestamp();
        let found = if next {
            tz.following(t).next().map(|tr| tr.timestamp())
        } else {
            tz.preceding(t).next().map(|tr| tr.timestamp())
        };
        match found.map(|ts| ts.to_zoned(tz.clone())) {
            Some(tr) => out.push(Some((&tr, &id))),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_zoned_add(x: ListSexp, duration: ListSexp, reject: bool) -> savvy::Result<savvy::Sexp> {
    let (xc, dc) = (ZonedCols::new(&x)?, DurationCols::new(&duration)?);
    let (x, d) = (xc.reader()?, dc.reader()?);
    let n = common_len(&[x.len(), d.len()])?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i, &mut cache)?, d.get(i)?) {
            (Some((z, id)), Some(span)) => {
                if reject {
                    check_reject(i, z.date(), span)?;
                }
                let r = z.checked_add(span).map_err(|e| elt_error(i, e))?;
                out.push(Some((&r, &id)));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_zoned_diff(
    x: ListSexp,
    y: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    since: bool,
) -> savvy::Result<savvy::Sexp> {
    let opts = DiffOpts::new(largest, smallest, increment, mode)?;
    let (xc, yc) = (ZonedCols::new(&x)?, ZonedCols::new(&y)?);
    let (a, b) = (xc.reader()?, yc.reader()?);
    let n = common_len(&[a.len(), b.len()])?;
    let (mut cache_a, mut cache_b) = (TzCache::default(), TzCache::default());
    let calendar = opts.largest.is_some_and(|u| u >= Unit::Day);
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i, &mut cache_a)?, b.get(i, &mut cache_b)?) {
            (Some((a, ida)), Some((b, idb))) => {
                let same_id = Rc::ptr_eq(&ida, &idb) || *ida == *idb;
                // Temporal's TimeZoneEquals, as in temporal_equals(): aliases
                // of UTC are one zone. jiff wants the same zone object for
                // calendar units, so `b` is viewed in `a`'s zone (same
                // offsets, hence the same exact time).
                let b = if same_id {
                    b
                } else if time_zones_equal(&ida, &idb) {
                    b.with_time_zone(a.time_zone().clone())
                } else if calendar {
                    return Err(elt_error(
                        i,
                        "time zones must match to compute a difference in calendar units",
                    ));
                } else {
                    b
                };
                let r = difference(&a, &b, &opts, since);
                out.push(Some(r.map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_zoned_round(
    x: ListSexp,
    smallest: &str,
    increment: f64,
    mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let cols = ZonedCols::new(&x)?;
    let x = cols.reader()?;
    let (unit, increment, mode) = (
        parse_unit(smallest)?,
        increment_i64(increment)?,
        parse_round_mode(mode)?,
    );
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some((z, id)) => {
                let r = round_zoned(&z, unit, increment, mode).map_err(|e| elt_error(i, e))?;
                out.push(Some((&r, &id)));
            }
            None => out.push(None),
        }
    }
    out.into_sexp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;
    use jiff::tz::TimeZone;
    use jiff::ToSpan;

    fn ny() -> TimeZone {
        TimeZone::get("America/New_York").unwrap()
    }

    #[test]
    fn dst_gap_and_overlap() {
        // 2019-03-10T02:30 does not exist in New York; 2019-11-03T01:30 is
        // ambiguous.
        let gap = date(2019, 3, 10).at(2, 30, 0, 0);
        let overlap = date(2019, 11, 3).at(1, 30, 0, 0);
        let none = None;
        let c = OffsetConflict::Reject;
        let z = zoned_from_civil(0, gap, ny(), none, c, Disambiguation::Compatible).unwrap();
        assert_eq!(z.to_string(), "2019-03-10T03:30:00-04:00[America/New_York]");
        let z = zoned_from_civil(0, gap, ny(), none, c, Disambiguation::Earlier).unwrap();
        assert_eq!(z.to_string(), "2019-03-10T01:30:00-05:00[America/New_York]");
        assert!(zoned_from_civil(0, gap, ny(), none, c, Disambiguation::Reject).is_err());
        let z = zoned_from_civil(0, overlap, ny(), none, c, Disambiguation::Compatible).unwrap();
        assert_eq!(z.to_string(), "2019-11-03T01:30:00-04:00[America/New_York]");
        let z = zoned_from_civil(0, overlap, ny(), none, c, Disambiguation::Later).unwrap();
        assert_eq!(z.to_string(), "2019-11-03T01:30:00-05:00[America/New_York]");
        // `prefer` keeps a matching offset in an overlap
        let z = zoned_from_civil(
            0,
            overlap,
            ny(),
            Some(Offset::from_seconds(-5 * 3600).unwrap()),
            OffsetConflict::PreferOffset,
            Disambiguation::Compatible,
        )
        .unwrap();
        assert_eq!(z.to_string(), "2019-11-03T01:30:00-05:00[America/New_York]");
    }

    #[test]
    fn calendar_arithmetic_across_dst() {
        let z = date(2020, 3, 7).at(12, 0, 0, 0).to_zoned(ny()).unwrap();
        // one day later is still noon, but only 23 hours elapsed
        let next = z.checked_add(1.day()).unwrap();
        assert_eq!(next.datetime(), date(2020, 3, 8).at(12, 0, 0, 0));
        assert_eq!(z.until(&next).unwrap().get_hours(), 23);
    }

    #[test]
    fn parse_offset_option() {
        let s = "2020-01-01T00:00+01:00[America/New_York]";
        assert!(parser().parse_zoned(s).is_err());
        let p = parser().offset_conflict(OffsetConflict::AlwaysTimeZone);
        assert_eq!(
            p.parse_zoned(s).unwrap().to_string(),
            "2020-01-01T00:00:00-05:00[America/New_York]"
        );
        let p = parser().offset_conflict(OffsetConflict::AlwaysOffset);
        assert_eq!(
            p.parse_zoned(s).unwrap().to_string(),
            "2019-12-31T18:00:00-05:00[America/New_York]"
        );
    }

    #[test]
    fn negative_timestamps_split_with_floor() {
        let t = Timestamp::new(-1, -500_000_000).unwrap();
        assert_eq!(crate::cols::timestamp_parts(t), (-2.0, 500_000_000));
    }
}
