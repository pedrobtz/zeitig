//! Temporal `Duration` (jiff `Span`).

use jiff::civil::DateTime;
use jiff::fmt::temporal::SpanParser;
use jiff::{SpanCompare, SpanRelativeTo, SpanRound, SpanTotal};
use savvy::{
    savvy, IntegerSexp, ListSexp, OwnedIntegerSexp, OwnedListSexp, OwnedRealSexp, OwnedStringSexp,
    StringSexp, TypedSexp,
};

use crate::cols::{common_len, elt_error, DateTimeIn, DurationIn, DurationOut};
use crate::opts::{increment_i64, parse_round_mode, parse_unit, parse_unit_auto};

static SPAN_PARSER: SpanParser = SpanParser::new();

/// The five integer columns of a plain date-time record.
pub(crate) struct DateTimeCols([IntegerSexp; 5]);

impl DateTimeCols {
    pub(crate) fn new(x: &ListSexp) -> savvy::Result<Self> {
        let mut cols = Vec::with_capacity(5);
        for k in 0..5 {
            match x.get_by_index(k).map(|s| s.into_typed()) {
                Some(TypedSexp::Integer(v)) => cols.push(v),
                _ => {
                    return Err(savvy::Error::new(
                        "internal error: date-time fields must be integer vectors",
                    ))
                }
            }
        }
        let cols: [IntegerSexp; 5] = cols
            .try_into()
            .map_err(|_| savvy::Error::new("internal error: expected five columns"))?;
        Ok(Self(cols))
    }

    pub(crate) fn reader(&self) -> savvy::Result<DateTimeIn<'_>> {
        let c = &self.0;
        DateTimeIn::new(&c[0], &c[1], &c[2], &c[3], &c[4])
    }
}

/// Optional `relative_to` column (plain date-times; plain dates are passed
/// as midnight). `None` means days are 24 hours and calendar units error.
pub(crate) struct Relative {
    cols: Option<DateTimeCols>,
}

impl Relative {
    pub(crate) fn new(x: Option<ListSexp>) -> savvy::Result<Self> {
        Ok(Self {
            cols: x.as_ref().map(DateTimeCols::new).transpose()?,
        })
    }

    pub(crate) fn len(&self) -> savvy::Result<Option<usize>> {
        match &self.cols {
            Some(c) => Ok(Some(c.reader()?.len())),
            None => Ok(None),
        }
    }

    /// The relative-to anchor for element `i`. `Ok(None)` when no
    /// `relative_to` was given; `Err` never for missing values: a missing
    /// anchor is reported by `is_na()`.
    pub(crate) fn anchor(&self, i: usize) -> savvy::Result<Anchor> {
        match &self.cols {
            None => Ok(Anchor::DaysAre24Hours),
            Some(c) => match c.reader()?.get(i)? {
                Some(dt) => Ok(Anchor::Civil(dt)),
                None => Ok(Anchor::Missing),
            },
        }
    }
}

pub(crate) enum Anchor {
    DaysAre24Hours,
    Civil(DateTime),
    Missing,
}

impl Anchor {
    fn relative(&self) -> Option<SpanRelativeTo<'static>> {
        match self {
            Anchor::DaysAre24Hours => Some(SpanRelativeTo::days_are_24_hours()),
            Anchor::Civil(dt) => Some((*dt).into()),
            Anchor::Missing => None,
        }
    }
}

fn check_len(n: usize, rel: &Relative) -> savvy::Result<()> {
    if let Some(m) = rel.len()? {
        common_len(&[n, m])?;
    }
    Ok(())
}

#[savvy]
fn rs_duration_validate(x: ListSexp) -> savvy::Result<savvy::Sexp> {
    let x = DurationIn::new(&x)?;
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
    let x = DurationIn::new(&x)?;
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
    let x = DurationIn::new(&x)?;
    let y = DurationIn::new(&y)?;
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
    let x = DurationIn::new(&x)?;
    let rel = Relative::new(relative)?;
    check_len(x.len(), &rel)?;
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
        let mut opts = SpanRound::new()
            .smallest(smallest)
            .increment(increment)
            .mode(mode)
            .relative(relative);
        if let Some(l) = largest {
            opts = opts.largest(l);
        }
        out.push(Some(span.round(opts).map_err(|e| elt_error(i, e))?));
    }
    out.into_sexp()
}

#[savvy]
fn rs_duration_total(
    x: ListSexp,
    unit: &str,
    relative: Option<ListSexp>,
) -> savvy::Result<savvy::Sexp> {
    let x = DurationIn::new(&x)?;
    let rel = Relative::new(relative)?;
    check_len(x.len(), &rel)?;
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
    let x = DurationIn::new(&x)?;
    let y = DurationIn::new(&y)?;
    let n = common_len(&[x.len(), y.len()])?;
    let rel = Relative::new(relative)?;
    check_len(n, &rel)?;
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
    let x = DurationIn::new(&x)?;
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
