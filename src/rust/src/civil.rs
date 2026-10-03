//! PlainDate, PlainTime and PlainDateTime (jiff `civil::{Date, Time, DateTime}`).

use jiff::civil::{Date, DateTime, Time};
use jiff::{Span, Unit};
use savvy::{
    savvy, IntegerSexp, NotAvailableValue, OwnedIntegerSexp, OwnedRealSexp, OwnedStringSexp,
    RealSexp, StringSexp,
};

use crate::cols::{
    common_len, elt_error, is_na_int, DateIn, DateOut, DateTimeIn, DateTimeOut, TimeIn, TimeOut,
};
use crate::ixdtf::{prepare, Kind};

fn range_error(i: usize, what: &str, value: i64, lo: i64, hi: i64) -> savvy::Error {
    elt_error(
        i,
        format!(
            "parameter '{what}' with value {value} is not in the required range of {lo}..={hi}"
        ),
    )
}

/// Validates (`reject`) or clamps (`constrain`) a field to `lo..=hi`.
fn regulate(
    i: usize,
    what: &str,
    value: i64,
    lo: i64,
    hi: i64,
    reject: bool,
) -> savvy::Result<i64> {
    if (lo..=hi).contains(&value) {
        Ok(value)
    } else if reject {
        Err(range_error(i, what, value, lo, hi))
    } else {
        Ok(value.clamp(lo, hi))
    }
}

/// Temporal `RegulateISODate`: year must be in range, month and day must be
/// positive; with `constrain` month is clamped to 12 and day to the length of
/// the month.
pub(crate) fn regulate_date(
    i: usize,
    year: i32,
    month: i32,
    day: i32,
    reject: bool,
) -> savvy::Result<Date> {
    let year = regulate(i, "year", year.into(), -9999, 9999, true)? as i16;
    let month = regulate(i, "month", month.into(), 1, 12, reject || month < 1)? as i8;
    let first = Date::new(year, month, 1).map_err(|e| elt_error(i, e))?;
    let dim = i64::from(first.days_in_month());
    let day = regulate(i, "day", day.into(), 1, dim, reject || day < 1)? as i8;
    Date::new(year, month, day).map_err(|e| elt_error(i, e))
}

/// Temporal `RegulateTime`: with `constrain` every field is clamped to its range.
#[allow(clippy::too_many_arguments)]
pub(crate) fn regulate_time(
    i: usize,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    reject: bool,
) -> savvy::Result<Time> {
    let h = regulate(i, "hour", hour.into(), 0, 23, reject)?;
    let mi = regulate(i, "minute", minute.into(), 0, 59, reject)?;
    let s = regulate(i, "second", second.into(), 0, 59, reject)?;
    let ms = regulate(i, "millisecond", millisecond.into(), 0, 999, reject)?;
    let us = regulate(i, "microsecond", microsecond.into(), 0, 999, reject)?;
    let ns = regulate(i, "nanosecond", nanosecond.into(), 0, 999, reject)?;
    let subsec = (ms * 1_000_000 + us * 1_000 + ns) as i32;
    Time::new(h as i8, mi as i8, s as i8, subsec).map_err(|e| elt_error(i, e))
}

fn any_na(xs: &[i32]) -> bool {
    xs.iter().any(|&x| is_na_int(x))
}

// ---------------------------------------------------------------------------
// PlainDate

#[savvy]
fn rs_plain_date_from_parts(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
    reject: bool,
) -> savvy::Result<savvy::Sexp> {
    let n = common_len(&[year.len(), month.len(), day.len()])?;
    let (y, m, d) = (year.as_slice(), month.as_slice(), day.as_slice());
    let mut out = DateOut::with_capacity(n);
    for i in 0..n {
        if any_na(&[y[i], m[i], d[i]]) {
            out.push(None);
        } else {
            out.push(Some(regulate_date(i, y[i], m[i], d[i], reject)?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_parse(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = DateOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if s.is_na() {
            out.push(None);
        } else {
            let s = prepare(s, Kind::Calendar).map_err(|e| elt_error(i, e))?;
            out.push(Some(s.parse::<Date>().map_err(|e| elt_error(i, e))?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_format(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
) -> savvy::Result<savvy::Sexp> {
    let x = DateIn::new(&year, &month, &day)?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(d) => out.set_elt(i, &d.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

/// Derived calendar fields of a date (Temporal `PlainDate` getters).
pub(crate) fn date_field(d: Date, field: &str) -> savvy::Result<i32> {
    Ok(match field {
        "day_of_week" => d.weekday().to_monday_one_offset().into(),
        "day_of_year" => d.day_of_year().into(),
        "week_of_year" => d.iso_week_date().week().into(),
        "year_of_week" => d.iso_week_date().year().into(),
        "days_in_month" => d.days_in_month().into(),
        "days_in_year" => d.days_in_year().into(),
        "in_leap_year" => d.in_leap_year().into(),
        _ => return Err(savvy::Error::new(format!("unknown field '{field}'"))),
    })
}

#[savvy]
fn rs_plain_date_field(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
    field: &str,
) -> savvy::Result<savvy::Sexp> {
    let x = DateIn::new(&year, &month, &day)?;
    let mut out = OwnedIntegerSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(d) => out.set_elt(i, date_field(d, field)?)?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

const UNIX_EPOCH: Date = Date::constant(1970, 1, 1);

#[savvy]
fn rs_plain_date_from_epoch_days(days: RealSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = DateOut::with_capacity(days.len());
    for (i, &v) in days.iter().enumerate() {
        if v.is_na() || !v.is_finite() {
            out.push(None);
            continue;
        }
        let v = v.floor();
        if v.abs() > 1e8 {
            return Err(elt_error(
                i,
                format!("{v} days is outside the supported date range"),
            ));
        }
        let span = Span::new()
            .try_days(v as i64)
            .map_err(|e| elt_error(i, e))?;
        out.push(Some(
            UNIX_EPOCH.checked_add(span).map_err(|e| elt_error(i, e))?,
        ));
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_to_epoch_days(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
) -> savvy::Result<savvy::Sexp> {
    let x = DateIn::new(&year, &month, &day)?;
    let mut out = OwnedRealSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(d) => {
                let span = UNIX_EPOCH
                    .until((Unit::Day, d))
                    .map_err(|e| elt_error(i, e))?;
                out.set_elt(i, f64::from(span.get_days()))?;
            }
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// ---------------------------------------------------------------------------
// PlainTime

#[savvy]
fn rs_plain_time_from_parts(
    hour: IntegerSexp,
    minute: IntegerSexp,
    second: IntegerSexp,
    millisecond: IntegerSexp,
    microsecond: IntegerSexp,
    nanosecond: IntegerSexp,
    reject: bool,
) -> savvy::Result<savvy::Sexp> {
    let n = common_len(&[
        hour.len(),
        minute.len(),
        second.len(),
        millisecond.len(),
        microsecond.len(),
        nanosecond.len(),
    ])?;
    let cols = [
        hour.as_slice(),
        minute.as_slice(),
        second.as_slice(),
        millisecond.as_slice(),
        microsecond.as_slice(),
        nanosecond.as_slice(),
    ];
    let mut out = TimeOut::with_capacity(n);
    for i in 0..n {
        let v: Vec<i32> = cols.iter().map(|c| c[i]).collect();
        if any_na(&v) {
            out.push(None);
        } else {
            out.push(Some(regulate_time(
                i, v[0], v[1], v[2], v[3], v[4], v[5], reject,
            )?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_time_parse(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = TimeOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if s.is_na() {
            out.push(None);
        } else {
            let s = prepare(s, Kind::Time).map_err(|e| elt_error(i, e))?;
            out.push(Some(s.parse::<Time>().map_err(|e| elt_error(i, e))?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_time_format(
    second_of_day: IntegerSexp,
    nanos: IntegerSexp,
) -> savvy::Result<savvy::Sexp> {
    let x = TimeIn::new(&second_of_day, &nanos)?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.set_elt(i, &t.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// ---------------------------------------------------------------------------
// PlainDateTime

#[savvy]
fn rs_plain_date_time_parse(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = DateTimeOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if s.is_na() {
            out.push(None);
        } else {
            let s = prepare(s, Kind::Calendar).map_err(|e| elt_error(i, e))?;
            out.push(Some(s.parse::<DateTime>().map_err(|e| elt_error(i, e))?));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_time_format(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
    second_of_day: IntegerSexp,
    nanos: IntegerSexp,
) -> savvy::Result<savvy::Sexp> {
    let x = DateTimeIn::new(&year, &month, &day, &second_of_day, &nanos)?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(dt) => out.set_elt(i, &dt.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrain_clamps_day_and_month() {
        let d = regulate_date(0, 2021, 2, 31, false).unwrap();
        assert_eq!(d, Date::constant(2021, 2, 28));
        let d = regulate_date(0, 2021, 13, 5, false).unwrap();
        assert_eq!(d, Date::constant(2021, 12, 5));
        assert!(regulate_date(0, 2021, 2, 31, true).is_err());
        assert!(regulate_date(0, 2021, 0, 1, false).is_err());
        assert!(regulate_date(0, 10000, 1, 1, false).is_err());
    }

    #[test]
    fn constrain_clamps_time() {
        let t = regulate_time(0, 24, 60, 60, 1000, 0, 0, false).unwrap();
        assert_eq!(t, Time::constant(23, 59, 59, 999_000_000));
        assert!(regulate_time(0, 24, 0, 0, 0, 0, 0, true).is_err());
    }

    #[test]
    fn display_matches_temporal() {
        assert_eq!(Time::constant(15, 23, 30, 0).to_string(), "15:23:30");
        assert_eq!(
            Time::constant(15, 23, 30, 500_000_000).to_string(),
            "15:23:30.5"
        );
        assert_eq!(
            Time::constant(15, 23, 30, 123_456_789).to_string(),
            "15:23:30.123456789"
        );
        assert_eq!(Date::constant(-1, 1, 1).to_string(), "-000001-01-01");
        assert_eq!(
            Date::constant(2020, 1, 1)
                .to_datetime(Time::midnight())
                .to_string(),
            "2020-01-01T00:00:00"
        );
    }

    #[test]
    fn parsing_follows_temporal() {
        assert!("2020-01-01T00:00Z".parse::<Date>().is_err());
        assert_eq!(
            "2019-11-18T15:23:30.123+01:00[Europe/Paris]"
                .parse::<Date>()
                .unwrap(),
            Date::constant(2019, 11, 18)
        );
        assert_eq!(
            "2019-11-18T15:23:30.123".parse::<Time>().unwrap(),
            Time::constant(15, 23, 30, 123_000_000)
        );
        assert_eq!(
            "15:23".parse::<Time>().unwrap(),
            Time::constant(15, 23, 0, 0)
        );
    }

    #[test]
    fn epoch_days_round_trip() {
        let d = UNIX_EPOCH.checked_add(Span::new().days(19_000)).unwrap();
        assert_eq!(UNIX_EPOCH.until((Unit::Day, d)).unwrap().get_days(), 19_000);
    }
}
