//! Temporal `Duration` (jiff `Span`).

use jiff::civil::DateTime;
use jiff::fmt::temporal::SpanParser;
use jiff::{RoundMode, SpanCompare, SpanRelativeTo, SpanRound, SpanTotal, Unit, Zoned};
use savvy::{
    savvy, ListSexp, OwnedIntegerSexp, OwnedListSexp, OwnedRealSexp, OwnedStringSexp, StringSexp,
};

use crate::arith::{half_even, unit_value};
use crate::cols::{
    common_len, elt_error, DateTimeCols, DateTimeIn, DurationCols, DurationOut, ZonedCols, ZonedIn,
};
use crate::opts::{increment_i64, parse_round_mode, parse_unit, parse_unit_auto};
use crate::tz::TzCache;

static SPAN_PARSER: SpanParser = SpanParser::new();

/// Optional `relative_to` columns: plain date-times (plain dates are passed
/// as midnight) or zoned date-times. Without it days are 24 hours and
/// calendar units error.
pub(crate) enum Relative {
    None,
    Civil(DateTimeCols),
    Zoned(ZonedCols),
}

impl Relative {
    pub(crate) fn new(x: Option<ListSexp>) -> savvy::Result<Self> {
        Ok(match x {
            None => Relative::None,
            Some(x) if x.len() == 3 => Relative::Zoned(ZonedCols::new(&x)?),
            Some(x) => Relative::Civil(DateTimeCols::new(&x)?),
        })
    }

    /// The reader over the columns, built once per call.
    pub(crate) fn reader(&self) -> savvy::Result<RelativeIn<'_>> {
        Ok(match self {
            Relative::None => RelativeIn::None,
            Relative::Civil(c) => RelativeIn::Civil(c.reader()?),
            Relative::Zoned(c) => RelativeIn::Zoned(c.reader()?, TzCache::default()),
        })
    }
}

pub(crate) enum RelativeIn<'a> {
    None,
    Civil(DateTimeIn<'a>),
    Zoned(ZonedIn<'a>, TzCache),
}

impl RelativeIn<'_> {
    /// Errors unless the relative-to column has length `n` (when given).
    fn check_len(&self, n: usize) -> savvy::Result<()> {
        let m = match self {
            RelativeIn::None => return Ok(()),
            RelativeIn::Civil(c) => c.len(),
            RelativeIn::Zoned(z, _) => z.len(),
        };
        common_len(&[n, m]).map(|_| ())
    }

    /// The relative-to anchor for element `i`.
    pub(crate) fn anchor(&mut self, i: usize) -> savvy::Result<Anchor> {
        Ok(match self {
            RelativeIn::None => Anchor::DaysAre24Hours,
            RelativeIn::Civil(c) => match c.get(i)? {
                Some(dt) => Anchor::Civil(dt),
                None => Anchor::Missing,
            },
            RelativeIn::Zoned(z, cache) => match z.get(i, cache)? {
                Some((z, _)) => Anchor::Zoned(Box::new(z)),
                None => Anchor::Missing,
            },
        })
    }
}

pub(crate) enum Anchor {
    DaysAre24Hours,
    Civil(DateTime),
    Zoned(Box<Zoned>),
    Missing,
}

impl Anchor {
    fn relative(&self) -> Option<SpanRelativeTo<'_>> {
        match self {
            Anchor::DaysAre24Hours => Some(SpanRelativeTo::days_are_24_hours()),
            Anchor::Civil(dt) => Some((*dt).into()),
            Anchor::Zoned(z) => Some(z.as_ref().into()),
            Anchor::Missing => None,
        }
    }
}

#[savvy]
fn rs_duration_validate(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = DurationCols::new(&x)?;
    let x = cols.reader()?;
    let mut out = DurationOut::with_capacity(x.len());
    for i in 0..x.len() {
        out.push(x.get(i)?);
    }
    out.into_sexp()
}

#[savvy]
fn rs_duration_parse(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = DurationOut::with_capacity(x.len());
    for (i, s) in x.iter().enumerate() {
        if crate::cols::is_na_str(s) {
            out.push(None);
        } else {
            out.push(Some(
                SPAN_PARSER.parse_span(s).map_err(|e| elt_error(i, e))?,
            ));
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_duration_format(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = DurationCols::new(&x)?;
    let x = cols.reader()?;
    let mut out = OwnedStringSexp::new(x.len())?;
    for i in 0..x.len() {
        match x.get(i)? {
            Some(s) => out.set_elt(i, &s.to_string())?,
            None => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// `Duration.prototype.add()`: days are 24 hours, calendar units error.
#[savvy]
fn rs_duration_add(x: ListSexp, y: ListSexp) -> savvy::Result<savvy::Sexp> {
    let (xc, yc) = (DurationCols::new(&x)?, DurationCols::new(&y)?);
    let (x, y) = (xc.reader()?, yc.reader()?);
    let n = common_len(&[x.len(), y.len()])?;
    let mut out = DurationOut::with_capacity(n);
    for i in 0..n {
        match (x.get(i)?, y.get(i)?) {
            (Some(a), Some(b)) => {
                let r = a
                    .checked_add((b, SpanRelativeTo::days_are_24_hours()))
                    .map_err(|e| elt_error(i, e))?;
                out.push(Some(r));
            }
            _ => out.push(None),
        }
    }
    out.into_sexp()
}

#[savvy]
fn rs_duration_round(
    x: ListSexp,
    largest: &str,
    smallest: &str,
    increment: f64,
    mode: &str,
    relative: Option<ListSexp>,
) -> savvy::Result<savvy::Sexp> {
    let cols = DurationCols::new(&x)?;
    let x = cols.reader()?;
    let rel = Relative::new(relative)?;
    let mut rel = rel.reader()?;
    rel.check_len(x.len())?;
    let largest = parse_unit_auto(largest)?;
    let smallest = parse_unit(smallest)?;
    let increment = increment_i64(increment)?;
    let mode = parse_round_mode(mode)?;
    let mut out = DurationOut::with_capacity(x.len());
    for i in 0..x.len() {
        let (span, anchor) = (x.get(i)?, rel.anchor(i)?);
        let (Some(span), Some(relative)) = (span, anchor.relative()) else {
            out.push(None);
            continue;
        };
        let round = |mode| {
            let mut opts = SpanRound::new()
                .smallest(smallest)
                .increment(increment)
                .mode(mode)
                .relative(relative);
            if let Some(l) = largest {
                opts = opts.largest(l);
            }
            span.round(opts).map_err(|e| elt_error(i, e))
        };
        // jiff sometimes resolves exact `halfEven` ties to calendar units or
        // days to the odd neighbour; Temporal picks the even one.
        let rounded = if mode == RoundMode::HalfEven && smallest >= Unit::Day {
            half_even(
                round,
                |s| unit_value(s, smallest).abs() / increment,
                |a, b| a.fieldwise() == b.fieldwise(),
            )?
        } else {
            round(mode)?
        };
        out.push(Some(rounded));
    }
    out.into_sexp()
}

#[savvy]
fn rs_duration_total(
    x: ListSexp,
    unit: &str,
    relative: Option<ListSexp>,
) -> savvy::Result<savvy::Sexp> {
    let cols = DurationCols::new(&x)?;
    let x = cols.reader()?;
    let rel = Relative::new(relative)?;
    let mut rel = rel.reader()?;
    rel.check_len(x.len())?;
    let unit = parse_unit(unit)?;
    let mut out = OwnedRealSexp::new(x.len())?;
    for i in 0..x.len() {
        let (span, anchor) = (x.get(i)?, rel.anchor(i)?);
        match (span, anchor.relative()) {
            (Some(span), Some(relative)) => {
                let total = span
                    .total(SpanTotal::from((unit, relative)))
                    .map_err(|e| elt_error(i, e))?;
                out.set_elt(i, total)?;
            }
            _ => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

#[savvy]
fn rs_duration_compare(
    x: ListSexp,
    y: ListSexp,
    relative: Option<ListSexp>,
) -> savvy::Result<savvy::Sexp> {
    let (xc, yc) = (DurationCols::new(&x)?, DurationCols::new(&y)?);
    let (x, y) = (xc.reader()?, yc.reader()?);
    let n = common_len(&[x.len(), y.len()])?;
    let rel = Relative::new(relative)?;
    let mut rel = rel.reader()?;
    rel.check_len(n)?;
    let mut out = OwnedIntegerSexp::new(n)?;
    for i in 0..n {
        let anchor = rel.anchor(i)?;
        match (x.get(i)?, y.get(i)?, anchor.relative()) {
            (Some(a), Some(b), Some(relative)) => {
                let ord = a
                    .compare(SpanCompare::from((b, relative)))
                    .map_err(|e| elt_error(i, e))?;
                out.set_elt(i, ord as i32)?;
            }
            _ => out.set_na(i)?,
        }
    }
    Ok(out.into())
}

// Sort key for `vec_proxy_compare()`: whole seconds and nanoseconds of the
// duration with 24-hour days. Calendar units are an error, as comparing
// them needs `relative_to`.
#[savvy]
fn rs_duration_sort_key(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let cols = DurationCols::new(&x)?;
    let x = cols.reader()?;
    let n = x.len();
    let mut secs = OwnedRealSexp::new(n)?;
    let mut nanos = OwnedIntegerSexp::new(n)?;
    for i in 0..n {
        match x.get(i)? {
            Some(span) => {
                let d = span
                    .to_duration(SpanRelativeTo::days_are_24_hours())
                    .map_err(|e| elt_error(i, e))?;
                secs.set_elt(i, d.as_secs() as f64)?;
                nanos.set_elt(i, d.subsec_nanos())?;
            }
            None => {
                secs.set_na(i)?;
                nanos.set_na(i)?;
            }
        }
    }
    let mut out = OwnedListSexp::new(2, true)?;
    out.set_name_and_value(0, "seconds", secs)?;
    out.set_name_and_value(1, "nanos", nanos)?;
    Ok(out.into())
}

#[cfg(test)]
pub(crate) fn tests_parse(s: &str) -> jiff::Span {
    SPAN_PARSER.parse_span(s).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cols::{span_from_fields, span_to_fields};
    use jiff::{Span, ToSpan};

    #[test]
    fn iso_strings_round_trip() {
        for s in [
            "P1Y2M3W4DT5H6M7.00800901S",
            "PT0S",
            "-P1D",
            "PT1.5S",
            "PT36H",
            "P1Y",
            "PT0.000000001S",
        ] {
            let span = SPAN_PARSER.parse_span(s).unwrap();
            assert_eq!(span.to_string(), s, "{s}");
        }
    }

    #[test]
    fn temporal_formatting() {
        assert_eq!(Span::new().to_string(), "PT0S");
        assert_eq!(1500.milliseconds().to_string(), "PT1.5S");
        assert_eq!((-90).minutes().to_string(), "-PT90M");
    }

    #[test]
    fn friendly_format_is_rejected() {
        assert!(SPAN_PARSER.parse_span("1 day").is_err());
    }

    #[test]
    fn fields_round_trip_and_sign() {
        let v = [1.0, 2.0, 0.0, 4.0, 0.0, 0.0, 7.0, 0.0, 0.0, 9.0];
        let span = span_from_fields(0, v).unwrap().unwrap();
        assert_eq!(span_to_fields(span), v);
        let neg = v.map(|x| -x);
        let span = span_from_fields(0, neg).unwrap().unwrap();
        assert_eq!(span.signum(), -1);
        assert_eq!(span_to_fields(span), neg);
        let mixed = [1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(span_from_fields(0, mixed).is_err());
        assert!(span_from_fields(0, [f64::NAN; 10]).unwrap().is_none());
        let frac = [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(span_from_fields(0, frac).is_err());
        let big = [20000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(span_from_fields(0, big).is_err());
    }
}
