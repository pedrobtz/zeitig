//! Temporal `Instant` (jiff `Timestamp`), `ZonedDateTime` (jiff `Zoned`) and
//! `Now`.

use jiff::civil::DateTime;
use jiff::fmt::temporal::DateTimeParser;
use jiff::tz::{Disambiguation, Offset, OffsetConflict};
use jiff::{RoundMode, Timestamp, TimestampRound, Zoned, ZonedRound};
use savvy::{
    savvy, IntegerSexp, ListSexp, OwnedIntegerSexp, OwnedListSexp, OwnedRealSexp, OwnedStringSexp,
    StringSexp,
};

use crate::arith::{check_reject, difference, round_time_like};
use crate::cols::{
    common_len, elt_error, is_na_int, is_na_str, DateTimeOut, DurationIn, DurationOut, InstantIn,
    InstantOut, ZonedIn, ZonedOut,
};
use crate::duration::DateTimeCols;
use crate::ixdtf::{prepare, Kind};
use crate::opts::{increment_i64, parse_round_mode, parse_unit, DiffOpts};
use crate::tz::{db, format_offset, parse_disambiguation, parse_offset_conflict, TzCache};

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
    let x = InstantIn::new(&x)?;
    let mut out = InstantOut::with_capacity(x.len());
    for i in 0..x.len() {
        out.push(x.get(i)?);
    }
    out.into_sexp()
}

#[savvy]
fn rs_instant_format(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let x = InstantIn::new(&x)?;
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
    let x = InstantIn::new(&x)?;
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
    let x = InstantIn::new(&x)?;
    let d = DurationIn::new(&duration)?;
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
    let (a, b) = (InstantIn::new(&x)?, InstantIn::new(&y)?);
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
    let x = InstantIn::new(&x)?;
    // Temporal rounds instants "as if positive" (RoundNumberToIncrementAsIfPositive):
    // `trunc` goes down even before 1970, while jiff rounds the signed epoch
    // value towards zero. Mapping each mode to its direction-fixed
    // counterpart gives Temporal's result for every instant.
    let mode = match parse_round_mode(mode)? {
        RoundMode::Trunc => RoundMode::Floor,
        RoundMode::Expand => RoundMode::Ceil,
        RoundMode::HalfTrunc => RoundMode::HalfFloor,
        RoundMode::HalfExpand => RoundMode::HalfCeil,
        m => m,
    };
    let opts = TimestampRound::new()
        .smallest(parse_unit(smallest)?)
        .increment(increment_i64(increment)?)
        .mode(mode);
    let mut out = InstantOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.push(Some(t.round(opts).map_err(|e| elt_error(i, e))?)),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_now() -> savvy::Result<savvy::Sexp> {
    let mut out = InstantOut::with_capacity(1);
    out.push(Some(Timestamp::now()));
    out.into_sexp()
}

// ---------------------------------------------------------------------------
// ZonedDateTime

fn str_col(x: &StringSexp) -> Vec<Option<String>> {
    x.iter()
        .map(|s| {
            if is_na_str(s) {
                None
            } else {
                Some(s.to_string())
            }
        })
        .collect()
}

// Instants (seconds, nanos) viewed in time zones -> zoned date-times. Also
// used by `with_time_zone()` and to validate/canonicalise identifiers.
#[savvy]
#[allow(clippy::needless_range_loop)]
fn rs_instant_to_zoned(x: ListSexp, time_zone: StringSexp) -> savvy::Result<savvy::Sexp> {
    let x = InstantIn::new(&x)?;
    let tz = str_col(&time_zone);
    let n = common_len(&[x.len(), tz.len()])?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, tz[i].as_deref()) {
            (Some(t), Some(id)) => {
                let zone = cache.get(i, id)?.0.clone();
                out.push(Some(&t.to_zoned(zone)));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

// Plain date-times in time zones -> zoned date-times, resolving gaps and
// overlaps with `disambiguation`. When `offset` (seconds, NA for none) is
// given it is reconciled with the zone using `offset_mode` (Temporal's
// `offset` option), as `ZonedDateTime.prototype.with()` does.
#[savvy]
fn rs_zoned_from_civil(
    x: ListSexp,
    time_zone: StringSexp,
    disambiguation: &str,
    offset: IntegerSexp,
    offset_mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let cols = DateTimeCols::new(&x)?;
    let x = cols.reader()?;
    let tz = str_col(&time_zone);
    let offsets = offset.as_slice();
    let n = common_len(&[x.len(), tz.len(), offsets.len()])?;
    let disambiguation = parse_disambiguation(disambiguation)?;
    let conflict = parse_offset_conflict(offset_mode)?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for i in 0..n {
        let (Some(dt), Some(id)) = (x.get(i)?, tz[i].as_deref()) else {
            out.push(None);
            continue;
        };
        let zone = cache.get(i, id)?.0.clone();
        let z = zoned_from_civil(i, dt, zone, offsets[i], conflict, disambiguation)?;
        out.push(Some(&z));
    }
    out.into_sexp()
}

fn zoned_from_civil(
    i: usize,
    dt: DateTime,
    zone: jiff::tz::TimeZone,
    offset: i32,
    conflict: OffsetConflict,
    disambiguation: Disambiguation,
) -> savvy::Result<Zoned> {
    let ambiguous = if is_na_int(offset) {
        zone.to_ambiguous_zoned(dt)
    } else {
        let off = Offset::from_seconds(offset).map_err(|e| elt_error(i, e))?;
        conflict
            .resolve(dt, off, zone)
            .map_err(|e| elt_error(i, e))?
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
    let mut out = ZonedOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if is_na_str(s) {
            out.push(None);
            continue;
        }
        let s = prepare(s, Kind::Zoned).map_err(|e| elt_error(i, e))?;
        let z = p.parse_zoned_with(db(), &*s).map_err(|e| elt_error(i, e))?;
        // Re-resolve the zone so fixed offsets and names are canonical and
        // POSIX TZ strings (not Temporal identifiers) are rejected. jiff
        // turns a `[+00:00]` annotation into UTC, which Temporal keeps apart.
        let id = match offset_annotation(&s) {
            Some(ann) => ann.to_string(),
            None => crate::tz::time_zone_id(z.time_zone()),
        };
        let (tz, _) = crate::tz::resolve_time_zone(&id).map_err(|e| elt_error(i, e))?;
        out.push(Some(&z.with_time_zone(tz)));
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
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some(z) => out.set_elt(i, &z.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// The wall-clock date-time of each element.
#[savvy]
fn rs_zoned_civil(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let mut out = DateTimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        out.push(x.get(i, &mut cache)?.map(|z| z.datetime()));
    }
    out.into_sexp()
}

#[savvy]
fn rs_zoned_offset(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let n = x.len();
    let mut secs = OwnedIntegerSexp::new(n)?;
    let mut text = OwnedStringSexp::new(n)?;
    for i in 0..n {
        match x.get(i, &mut cache)? {
            Some(z) => {
                let s = z.offset().seconds();
                secs.set_elt(i, s)?;
                text.set_elt(i, &format_offset(s))?;
            }
            None => {
                secs.set_na(i)?;
                text.set_na(i)?;
            }
        }
    }
    let mut out = OwnedListSexp::new(2, true)?;
    out.set_name_and_value(0, "seconds", secs)?;
    out.set_name_and_value(1, "string", text)?;
    Ok(out.into())
}

#[savvy]
fn rs_zoned_hours_in_day(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let mut out = OwnedRealSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some(z) => {
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
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some(z) => out.push(Some(&z.start_of_day().map_err(|e| elt_error(i, e))?)),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

// The next or previous UTC offset transition, `NA` when there is none.
#[savvy]
fn rs_zoned_transition(x: ListSexp, next: bool) -> savvy::Result<savvy::Sexp> {
    let x = ZonedIn::new(&x)?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        let Some(z) = x.get(i, &mut cache)? else {
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
        out.push(found.map(|ts| ts.to_zoned(tz.clone())).as_ref());
    }
    out.into_sexp()
}

#[savvy]
fn rs_zoned_add(x: ListSexp, duration: ListSexp, reject: bool) -> savvy::Result<savvy::Sexp> {
    let x = ZonedIn::new(&x)?;
    let d = DurationIn::new(&duration)?;
    let n = common_len(&[x.len(), d.len()])?;
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i, &mut cache)?, d.get(i)?) {
            (Some(z), Some(span)) => {
                if reject {
                    check_reject(i, z.date(), span)?;
                }
                out.push(Some(&z.checked_add(span).map_err(|e| elt_error(i, e))?));
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
    let (a, b) = (ZonedIn::new(&x)?, ZonedIn::new(&y)?);
    let n = common_len(&[a.len(), b.len()])?;
    let mut cache = TzCache::default();
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i, &mut cache)?, b.get(i, &mut cache)?) {
            (Some(a), Some(b)) => {
                let calendar = opts.largest.is_some_and(|u| u >= jiff::Unit::Day);
                if calendar && a.time_zone() != b.time_zone() {
                    return Err(elt_error(
                        i,
                        "time zones must match to compute a difference in calendar units",
                    ));
                }
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
    let x = ZonedIn::new(&x)?;
    let (unit, increment, mode) = (
        parse_unit(smallest)?,
        increment_i64(increment)?,
        parse_round_mode(mode)?,
    );
    let opts = ZonedRound::new().smallest(unit).increment(increment);
    let mut cache = TzCache::default();
    let mut out = ZonedOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i, &mut cache)? {
            Some(z) => out.push(Some(
                &round_time_like(
                    unit,
                    increment,
                    mode,
                    |m| z.round(opts.mode(m)),
                    |z| z.time(),
                )
                .map_err(|e| elt_error(i, e))?,
            )),
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
        let none = i32::MIN;
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
            -5 * 3600,
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
