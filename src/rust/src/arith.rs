//! Arithmetic on PlainDate, PlainTime and PlainDateTime: `add()`,
//! `until()`/`since()` and `round()`.

use jiff::civil::{
    Date, DateDifference, DateTimeDifference, DateTimeRound, TimeDifference, TimeRound,
};
use jiff::Span;
use savvy::{savvy, IntegerSexp, ListSexp};

use crate::cols::{
    common_len, elt_error, DateIn, DateOut, DateTimeOut, DurationIn, DurationOut, TimeIn, TimeOut,
};
use crate::duration::DateTimeCols;
use crate::opts::{increment_i64, parse_round_mode, parse_unit, DiffOpts};

/// Temporal `overflow: "reject"` for date arithmetic: adding the years and
/// months of `span` must not move the day of month (jiff always constrains).
pub(crate) fn check_reject(i: usize, date: Date, span: Span) -> savvy::Result<()> {
    if span.get_years() == 0 && span.get_months() == 0 {
        return Ok(());
    }
    let ym = Span::new()
        .years(span.get_years())
        .months(span.get_months());
    let mid = date.checked_add(ym).map_err(|e| elt_error(i, e))?;
    if mid.day() != date.day() {
        return Err(elt_error(
            i,
            format!(
                "day {} is out of range for {:04}-{:02} (overflow = \"reject\")",
                date.day(),
                mid.year(),
                mid.month()
            ),
        ));
    }
    Ok(())
}

#[savvy]
fn rs_plain_date_add(
    year: IntegerSexp,
    month: IntegerSexp,
    day: IntegerSexp,
    duration: ListSexp,
    reject: bool,
) -> savvy::Result<savvy::Sexp> {
    let x = DateIn::new(&year, &month, &day)?;
    let d = DurationIn::new(&duration)?;
    let n = common_len(&[x.len(), d.len()])?;
    let mut out = DateOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, d.get(i)?) {
            (Some(date), Some(span)) => {
                if reject {
                    check_reject(i, date, span)?;
                }
                out.push(Some(date.checked_add(span).map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_time_add(
    second_of_day: IntegerSexp,
    nanos: IntegerSexp,
    duration: ListSexp,
) -> savvy::Result<savvy::Sexp> {
    let x = TimeIn::new(&second_of_day, &nanos)?;
    let d = DurationIn::new(&duration)?;
    let n = common_len(&[x.len(), d.len()])?;
    let mut out = TimeOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, d.get(i)?) {
            // Temporal: calendar units are ignored and the time wraps.
            (Some(t), Some(span)) => out.push(Some(t.wrapping_add(span))),
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_time_add(
    x: ListSexp,
    duration: ListSexp,
    reject: bool,
) -> savvy::Result<savvy::Sexp> {
    let cols = DateTimeCols::new(&x)?;
    let x = cols.reader()?;
    let d = DurationIn::new(&duration)?;
    let n = common_len(&[x.len(), d.len()])?;
    let mut out = DateTimeOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, d.get(i)?) {
            (Some(dt), Some(span)) => {
                if reject {
                    check_reject(i, dt.date(), span)?;
                }
                out.push(Some(dt.checked_add(span).map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_plain_date_diff(
    x: ListSexp,
    y: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    since: bool,
) -> savvy::Result<savvy::Sexp> {
    let opts = DiffOpts::new(largest, smallest, increment, mode)?;
    let (xc, yc) = (date_cols(&x)?, date_cols(&y)?);
    let a = DateIn::new(&xc[0], &xc[1], &xc[2])?;
    let b = DateIn::new(&yc[0], &yc[1], &yc[2])?;
    let n = common_len(&[a.len(), b.len()])?;
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i)?, b.get(i)?) {
            (Some(a), Some(b)) => {
                let mut diff = DateDifference::new(b)
                    .smallest(opts.smallest)
                    .increment(opts.increment)
                    .mode(opts.mode);
                if let Some(l) = opts.largest {
                    diff = diff.largest(l);
                }
                let r = if since { a.since(diff) } else { a.until(diff) };
                out.push(Some(r.map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

fn date_cols(x: &ListSexp) -> savvy::Result<[IntegerSexp; 3]> {
    int_cols::<3>(x)
}

fn int_cols<const N: usize>(x: &ListSexp) -> savvy::Result<[IntegerSexp; N]> {
    let mut cols = Vec::with_capacity(N);
    for k in 0..N {
        match x.get_by_index(k).map(|s| s.into_typed()) {
            Some(savvy::TypedSexp::Integer(v)) => cols.push(v),
            _ => {
                return Err(savvy::Error::new(
                    "internal error: record fields must be integer vectors",
                ))
            }
        }
    }
    cols.try_into()
        .map_err(|_| savvy::Error::new("internal error: wrong number of record fields"))
}

#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_plain_time_diff(
    x: ListSexp,
    y: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    since: bool,
) -> savvy::Result<savvy::Sexp> {
    let opts = DiffOpts::new(largest, smallest, increment, mode)?;
    let (xc, yc) = (int_cols::<2>(&x)?, int_cols::<2>(&y)?);
    let a = TimeIn::new(&xc[0], &xc[1])?;
    let b = TimeIn::new(&yc[0], &yc[1])?;
    let n = common_len(&[a.len(), b.len()])?;
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i)?, b.get(i)?) {
            (Some(a), Some(b)) => {
                let mut diff = TimeDifference::new(b)
                    .smallest(opts.smallest)
                    .increment(opts.increment)
                    .mode(opts.mode);
                if let Some(l) = opts.largest {
                    diff = diff.largest(l);
                }
                let r = if since { a.since(diff) } else { a.until(diff) };
                out.push(Some(r.map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
#[allow(clippy::too_many_arguments)]
fn rs_plain_date_time_diff(
    x: ListSexp,
    y: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    since: bool,
) -> savvy::Result<savvy::Sexp> {
    let opts = DiffOpts::new(largest, smallest, increment, mode)?;
    let (xc, yc) = (DateTimeCols::new(&x)?, DateTimeCols::new(&y)?);
    let (a, b) = (xc.reader()?, yc.reader()?);
    let n = common_len(&[a.len(), b.len()])?;
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (a.get(i)?, b.get(i)?) {
            (Some(a), Some(b)) => {
                let mut diff = DateTimeDifference::new(b)
                    .smallest(opts.smallest)
                    .increment(opts.increment)
                    .mode(opts.mode);
                if let Some(l) = opts.largest {
                    diff = diff.largest(l);
                }
                let r = if since { a.since(diff) } else { a.until(diff) };
                out.push(Some(r.map_err(|e| elt_error(i, e))?));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_time_round(
    second_of_day: IntegerSexp,
    nanos: IntegerSexp,
    smallest: &str,
    increment: f64,
    mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let x = TimeIn::new(&second_of_day, &nanos)?;
    let opts = TimeRound::new()
        .smallest(parse_unit(smallest)?)
        .increment(increment_i64(increment)?)
        .mode(parse_round_mode(mode)?);
    let mut out = TimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.push(Some(t.round(opts).map_err(|e| elt_error(i, e))?)),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_plain_date_time_round(
    x: ListSexp,
    smallest: &str,
    increment: f64,
    mode: &str,
) -> savvy::Result<savvy::Sexp> {
    let cols = DateTimeCols::new(&x)?;
    let x = cols.reader()?;
    let opts = DateTimeRound::new()
        .smallest(parse_unit(smallest)?)
        .increment(increment_i64(increment)?)
        .mode(parse_round_mode(mode)?);
    let mut out = DateTimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(dt) => out.push(Some(dt.round(opts).map_err(|e| elt_error(i, e))?)),
            None => out.push(None),
        }
    }
    out.into_sexp()
}

#[cfg(test)]
mod tests {
    use jiff::civil::{date, time, Date, DateTime, Time};
    use jiff::{RoundMode, Span, ToSpan, Unit};

    /// Small deterministic generator so the property tests need no extra
    /// crate (every dependency is vendored into the CRAN tarball).
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }

        fn range(&mut self, lo: i64, hi: i64) -> i64 {
            lo + (self.next() % ((hi - lo + 1) as u64)) as i64
        }

        fn date(&mut self) -> Date {
            let y = self.range(1600, 2400) as i16;
            let m = self.range(1, 12) as i8;
            let first = date(y, m, 1);
            let d = self.range(1, first.days_in_month().into()) as i8;
            date(y, m, d)
        }

        fn time(&mut self) -> Time {
            Time::new(
                self.range(0, 23) as i8,
                self.range(0, 59) as i8,
                self.range(0, 59) as i8,
                self.range(0, 999_999_999) as i32,
            )
            .unwrap()
        }

        fn datetime(&mut self) -> DateTime {
            self.date().to_datetime(self.time())
        }
    }

    #[test]
    fn date_plus_difference_is_identity() {
        let mut g = Lcg(1);
        for largest in [Unit::Year, Unit::Month, Unit::Week, Unit::Day] {
            for _ in 0..2000 {
                let (a, b) = (g.date(), g.date());
                let d = a.until((largest, b)).unwrap();
                assert_eq!(a.checked_add(d).unwrap(), b, "{a} + {d} != {b}");
                // Temporal: a.since(b) is the negation of a.until(b)
                let s = a.since((largest, b)).unwrap();
                assert_eq!(s, d.negate().fieldwise());
            }
        }
    }

    #[test]
    fn datetime_plus_difference_is_identity() {
        let mut g = Lcg(2);
        for largest in [Unit::Year, Unit::Month, Unit::Day, Unit::Hour, Unit::Second] {
            for _ in 0..2000 {
                let (a, b) = (g.datetime(), g.datetime());
                let d = a.until((largest, b)).unwrap();
                assert_eq!(a.checked_add(d).unwrap(), b, "{a} + {d} != {b}");
            }
        }
    }

    #[test]
    fn time_plus_difference_is_identity() {
        let mut g = Lcg(3);
        for _ in 0..5000 {
            let (a, b) = (g.time(), g.time());
            let d = a.until(b).unwrap();
            assert_eq!(a.wrapping_add(d), b);
        }
    }

    #[test]
    fn strings_round_trip() {
        let mut g = Lcg(4);
        for _ in 0..5000 {
            let dt = g.datetime();
            assert_eq!(dt.to_string().parse::<DateTime>().unwrap(), dt);
            let d = dt.date();
            assert_eq!(d.to_string().parse::<Date>().unwrap(), d);
            let t = dt.time();
            assert_eq!(t.to_string().parse::<Time>().unwrap(), t);
            let span = dt.since((Unit::Year, g.datetime())).unwrap();
            let parsed = super::super::duration::tests_parse(&span.to_string());
            assert_eq!(parsed.fieldwise(), span.fieldwise());
        }
    }

    #[test]
    fn temporal_examples() {
        // Temporal docs: PlainDate.prototype.add
        assert_eq!(
            date(2021, 1, 31).checked_add(1.month()).unwrap(),
            date(2021, 2, 28)
        );
        assert!(super::check_reject(0, date(2021, 1, 31), 1.month()).is_err());
        assert!(super::check_reject(0, date(2021, 1, 15), 1.month()).is_ok());
        // time units are balanced into whole days
        assert_eq!(
            date(2021, 1, 1).checked_add(36.hours()).unwrap(),
            date(2021, 1, 2)
        );
        // PlainTime wraps and ignores calendar units
        assert_eq!(
            time(23, 0, 0, 0).wrapping_add(2.hours().days(3)),
            time(1, 0, 0, 0)
        );
        // PlainDate.until default largest unit is days
        assert_eq!(
            date(2006, 8, 24)
                .until(date(2019, 1, 31))
                .unwrap()
                .fieldwise(),
            4543.days().fieldwise()
        );
        // PlainTime.round
        assert_eq!(
            time(19, 39, 9, 68_346_205)
                .round(jiff::civil::TimeRound::new().smallest(Unit::Hour))
                .unwrap(),
            time(20, 0, 0, 0)
        );
        assert_eq!(
            time(19, 39, 9, 68_346_205)
                .round(
                    jiff::civil::TimeRound::new()
                        .smallest(Unit::Minute)
                        .increment(15)
                        .mode(RoundMode::Floor)
                )
                .unwrap(),
            time(19, 30, 0, 0)
        );
        let _ = Span::new();
    }
}
