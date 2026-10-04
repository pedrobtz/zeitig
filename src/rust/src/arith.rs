//! Arithmetic on PlainDate, PlainTime and PlainDateTime: `add()`,
//! `until()`/`since()` and `round()`.

use jiff::civil::{
    Date, DateDifference, DateTime, DateTimeDifference, DateTimeRound, Time, TimeDifference,
    TimeRound,
};
use jiff::{RoundMode, Span, Timestamp, TimestampDifference, Unit, Zoned, ZonedDifference};
use savvy::{savvy, IntegerSexp, ListSexp};

use crate::cols::{
    common_len, elt_error, DateCols, DateIn, DateOut, DateTimeCols, DateTimeOut, DurationCols,
    DurationOut, TimeCols, TimeIn, TimeOut,
};
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

/// A Temporal value with `until()`: the shared implementation of
/// `until()`/`since()` for every type.
pub(crate) trait Point: Sized {
    fn until_with(
        &self,
        other: &Self,
        largest: Option<Unit>,
        smallest: Unit,
        increment: i64,
        mode: RoundMode,
    ) -> Result<Span, jiff::Error>;
    fn add_weeks(&self, weeks: i64) -> Result<Self, jiff::Error>;
}

macro_rules! civil_point {
    ($t:ty, $diff:ident) => {
        impl Point for $t {
            fn until_with(
                &self,
                other: &Self,
                largest: Option<Unit>,
                smallest: Unit,
                increment: i64,
                mode: RoundMode,
            ) -> Result<Span, jiff::Error> {
                let mut d = $diff::new(*other)
                    .smallest(smallest)
                    .increment(increment)
                    .mode(mode);
                if let Some(l) = largest {
                    d = d.largest(l);
                }
                self.until(d)
            }
            fn add_weeks(&self, weeks: i64) -> Result<Self, jiff::Error> {
                self.checked_add(Span::new().try_weeks(weeks)?)
            }
        }
    };
}

civil_point!(Date, DateDifference);
civil_point!(DateTime, DateTimeDifference);
civil_point!(Time, TimeDifference);
civil_point!(Timestamp, TimestampDifference);

impl Point for Zoned {
    fn until_with(
        &self,
        other: &Self,
        largest: Option<Unit>,
        smallest: Unit,
        increment: i64,
        mode: RoundMode,
    ) -> Result<Span, jiff::Error> {
        let mut d = ZonedDifference::new(other)
            .smallest(smallest)
            .increment(increment)
            .mode(mode);
        if let Some(l) = largest {
            d = d.largest(l);
        }
        self.until(d)
    }
    fn add_weeks(&self, weeks: i64) -> Result<Self, jiff::Error> {
        self.checked_add(Span::new().try_weeks(weeks)?)
    }
}

/// Temporal's `a.until(b)` (or `a.since(b)`), with workarounds where jiff's
/// rounding differs from Temporal's:
///
/// * `since` is the negation of `until` with the negated rounding mode
///   (jiff's `since()` measures from `b`, which differs for calendar units);
/// * `halfEven` to a calendar unit or days resolves exact ties to the even
///   neighbour (jiff sometimes picks the odd one);
/// * with `largest_unit = "week"`, a day rounding increment applies to the days
///   left after whole weeks, not to the total number of days.
pub(crate) fn difference<P: Point>(
    a: &P,
    b: &P,
    o: &DiffOpts,
    since: bool,
) -> Result<Span, jiff::Error> {
    let span = rounded_until(a, b, o, o.mode_for(since))?;
    Ok(if since { span.negate() } else { span })
}

fn rounded_until<P: Point>(
    a: &P,
    b: &P,
    o: &DiffOpts,
    mode: RoundMode,
) -> Result<Span, jiff::Error> {
    if mode == RoundMode::HalfEven && o.smallest >= Unit::Day {
        return half_even(
            |m| rounded_until(a, b, o, m),
            |s| unit_value(s, o.smallest).abs() / o.increment,
            |x, y| x.fieldwise() == y.fieldwise(),
        );
    }
    if o.largest == Some(Unit::Week) && o.smallest == Unit::Day && o.increment > 1 {
        let weeks = i64::from(
            a.until_with(b, Some(Unit::Week), Unit::Day, 1, RoundMode::Trunc)?
                .get_weeks(),
        );
        let anchor = a.add_weeks(weeks)?;
        let days = i64::from(
            anchor
                .until_with(b, Some(Unit::Day), Unit::Day, o.increment, mode)?
                .get_days(),
        );
        // Temporal's BubbleRelativeDuration: a full week of rounded days
        // becomes one more week.
        return if days.abs() >= 7 {
            Span::new().try_weeks(weeks + days.signum())
        } else {
            Span::new().try_weeks(weeks)?.try_days(days)
        };
    }
    a.until_with(b, o.largest, o.smallest, o.increment, mode)
}

/// The value of one unit's field of a span.
pub(crate) fn unit_value(span: &Span, unit: Unit) -> i64 {
    match unit {
        Unit::Year => span.get_years().into(),
        Unit::Month => span.get_months().into(),
        Unit::Week => span.get_weeks().into(),
        Unit::Day => span.get_days().into(),
        Unit::Hour => span.get_hours().into(),
        Unit::Minute => span.get_minutes(),
        Unit::Second => span.get_seconds(),
        Unit::Millisecond => span.get_milliseconds(),
        Unit::Microsecond => span.get_microseconds(),
        Unit::Nanosecond => span.get_nanoseconds(),
    }
}

/// Temporal's `halfEven` built from jiff's `halfExpand` and `halfTrunc`, for
/// the cases where jiff may resolve an exact tie to the odd neighbour.
///
/// The two modes agree except at a tie, where they are one increment apart
/// and Temporal picks the one whose rounded quantity (`q`, in increments) is
/// even. `halfExpand` is computed first: when its quantity is even and
/// non-zero it is the answer whether or not this is a tie, so the second
/// rounding is only needed for odd quantities and for zero (which may be a
/// carry into the next larger unit, e.g. 6.5 days rounded up to 1 week).
pub(crate) fn half_even<T, E>(
    round: impl Fn(RoundMode) -> Result<T, E>,
    q: impl Fn(&T) -> i64,
    same: impl Fn(&T, &T) -> bool,
) -> Result<T, E> {
    let hi = round(RoundMode::HalfExpand)?;
    let qh = q(&hi);
    if qh != 0 && qh % 2 == 0 {
        return Ok(hi);
    }
    let lo = round(RoundMode::HalfTrunc)?;
    if same(&lo, &hi) || q(&lo) % 2 == 0 {
        Ok(lo)
    } else {
        Ok(hi)
    }
}

/// Rounds a wall-clock value as Temporal's `RoundTime` does. For `halfEven`,
/// Temporal decides a tie by the parity of the rounded unit's own field (the
/// quantity below the next larger unit; nothing for days), while jiff uses the
/// whole time of day. A tie is where `halfTrunc` and `halfExpand` disagree.
pub(crate) fn round_time_like<T: PartialEq>(
    unit: Unit,
    increment: i64,
    mode: RoundMode,
    round: impl Fn(RoundMode) -> Result<T, jiff::Error>,
    time: impl Fn(&T) -> Time,
) -> Result<T, jiff::Error> {
    if mode != RoundMode::HalfEven {
        return round(mode);
    }
    let field = |x: &T| {
        let t = time(x);
        let v = match unit {
            Unit::Hour => i64::from(t.hour()),
            Unit::Minute => i64::from(t.minute()),
            Unit::Second => i64::from(t.second()),
            Unit::Millisecond => i64::from(t.millisecond()),
            Unit::Microsecond => i64::from(t.microsecond()),
            Unit::Nanosecond => i64::from(t.nanosecond()),
            _ => 0,
        };
        v / increment
    };
    half_even(round, field, |a, b| a == b)
}

/// `PlainTime.prototype.round()` for one value.
pub(crate) fn round_time(
    t: Time,
    unit: Unit,
    increment: i64,
    mode: RoundMode,
) -> Result<Time, jiff::Error> {
    let opts = TimeRound::new().smallest(unit).increment(increment);
    round_time_like(unit, increment, mode, |m| t.round(opts.mode(m)), |t| *t)
}

/// `PlainDateTime.prototype.round()` for one value.
pub(crate) fn round_datetime(
    dt: DateTime,
    unit: Unit,
    increment: i64,
    mode: RoundMode,
) -> Result<DateTime, jiff::Error> {
    let opts = DateTimeRound::new().smallest(unit).increment(increment);
    round_time_like(
        unit,
        increment,
        mode,
        |m| dt.round(opts.mode(m)),
        |dt| dt.time(),
    )
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
    let dc = DurationCols::new(&duration)?;
    let d = dc.reader()?;
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
    let dc = DurationCols::new(&duration)?;
    let d = dc.reader()?;
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
    let dc = DurationCols::new(&duration)?;
    let d = dc.reader()?;
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
    let (xc, yc) = (DateCols::new(&x)?, DateCols::new(&y)?);
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
    let (xc, yc) = (TimeCols::new(&x)?, TimeCols::new(&y)?);
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
                let r = difference(&a, &b, &opts, since);
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
    let (unit, increment, mode) = (
        parse_unit(smallest)?,
        increment_i64(increment)?,
        parse_round_mode(mode)?,
    );
    let mut out = TimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(t) => out.push(Some(
                round_time(t, unit, increment, mode).map_err(|e| elt_error(i, e))?,
            )),
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
    let (unit, increment, mode) = (
        parse_unit(smallest)?,
        increment_i64(increment)?,
        parse_round_mode(mode)?,
    );
    let mut out = DateTimeOut::with_capacity(x.len());
    for i in 0..x.len() {
        match x.get(i)? {
            Some(dt) => out.push(Some(
                round_datetime(dt, unit, increment, mode).map_err(|e| elt_error(i, e))?,
            )),
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

    /// The pre-shortcut `halfEven`: always round both ways, then pick by the
    /// parity of the `halfTrunc` result.
    fn half_even_reference<T>(
        round: impl Fn(RoundMode) -> T,
        q: impl Fn(&T) -> i64,
        same: impl Fn(&T, &T) -> bool,
    ) -> T {
        let (lo, hi) = (round(RoundMode::HalfTrunc), round(RoundMode::HalfExpand));
        if same(&lo, &hi) || q(&lo) % 2 == 0 {
            lo
        } else {
            hi
        }
    }

    #[test]
    fn half_even_shortcut_matches_two_roundings() {
        use super::{round_time, rounded_until, unit_value};
        use crate::opts::DiffOpts;
        let mut g = Lcg(5);
        let units = [
            (Unit::Year, Unit::Year),
            (Unit::Year, Unit::Month),
            (Unit::Month, Unit::Day),
            (Unit::Week, Unit::Day),
            (Unit::Day, Unit::Day),
        ];
        let mut ties = 0;
        for _ in 0..1500 {
            // Whole days (ties for even increments) and date-times half a day
            // apart (ties when rounding to days).
            let (da, db) = (g.date(), g.date());
            let (ta, tb) = (da.at(0, 0, 0, 0), db.at(12, 0, 0, 0));
            for (largest, smallest) in units {
                for increment in [1, 2, 4] {
                    let o = DiffOpts {
                        largest: Some(largest),
                        smallest,
                        increment,
                        mode: RoundMode::HalfEven,
                    };
                    let q = |s: &Span| unit_value(s, smallest).abs() / increment;
                    let same = |x: &Span, y: &Span| x.fieldwise() == y.fieldwise();
                    let new = rounded_until(&da, &db, &o, RoundMode::HalfEven).unwrap();
                    let old =
                        half_even_reference(|m| rounded_until(&da, &db, &o, m).unwrap(), q, same);
                    assert_eq!(new.fieldwise(), old.fieldwise(), "{da} {db} {largest:?}");
                    let new = rounded_until(&ta, &tb, &o, RoundMode::HalfEven).unwrap();
                    let lo = rounded_until(&ta, &tb, &o, RoundMode::HalfTrunc).unwrap();
                    let hi = rounded_until(&ta, &tb, &o, RoundMode::HalfExpand).unwrap();
                    ties += usize::from(!same(&lo, &hi));
                    let old =
                        half_even_reference(|m| rounded_until(&ta, &tb, &o, m).unwrap(), q, same);
                    assert_eq!(new.fieldwise(), old.fieldwise(), "{ta} {tb} {largest:?}");
                }
            }
        }
        assert!(ties > 1000, "too few ties exercised: {ties}");
        for _ in 0..3000 {
            // hh:mm:30 is a tie when rounding to minutes, and so on.
            let t = time(g.range(0, 23) as i8, g.range(0, 59) as i8, 30, 500_000_000);
            for (unit, increment) in [
                (Unit::Hour, 1),
                (Unit::Minute, 1),
                (Unit::Minute, 2),
                (Unit::Second, 1),
                (Unit::Millisecond, 250),
            ] {
                let field = |x: &Time| {
                    let v = match unit {
                        Unit::Hour => i64::from(x.hour()),
                        Unit::Minute => i64::from(x.minute()),
                        Unit::Second => i64::from(x.second()),
                        _ => i64::from(x.millisecond()),
                    };
                    v / increment
                };
                let new = round_time(t, unit, increment, RoundMode::HalfEven).unwrap();
                let old = half_even_reference(
                    |m| round_time(t, unit, increment, m).unwrap(),
                    field,
                    |a, b| a == b,
                );
                assert_eq!(new, old, "{t} {unit:?} {increment}");
            }
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
