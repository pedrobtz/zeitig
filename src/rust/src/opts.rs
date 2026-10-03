//! Temporal option strings -> jiff enums. R validates the spelling first
//! (`rlang::arg_match()`), so an error here means a bug in the R caller.

use jiff::{RoundMode, Unit};

pub(crate) fn parse_unit(x: &str) -> savvy::Result<Unit> {
    Ok(match x {
        "year" => Unit::Year,
        "month" => Unit::Month,
        "week" => Unit::Week,
        "day" => Unit::Day,
        "hour" => Unit::Hour,
        "minute" => Unit::Minute,
        "second" => Unit::Second,
        "millisecond" => Unit::Millisecond,
        "microsecond" => Unit::Microsecond,
        "nanosecond" => Unit::Nanosecond,
        _ => return Err(savvy::Error::new(format!("invalid unit '{x}'"))),
    })
}

/// `"auto"` (Temporal's default) maps to `None`, letting jiff pick.
pub(crate) fn parse_unit_auto(x: &str) -> savvy::Result<Option<Unit>> {
    if x == "auto" {
        Ok(None)
    } else {
        parse_unit(x).map(Some)
    }
}

pub(crate) fn parse_round_mode(x: &str) -> savvy::Result<RoundMode> {
    Ok(match x {
        "ceil" => RoundMode::Ceil,
        "floor" => RoundMode::Floor,
        "expand" => RoundMode::Expand,
        "trunc" => RoundMode::Trunc,
        "halfCeil" => RoundMode::HalfCeil,
        "halfFloor" => RoundMode::HalfFloor,
        "halfExpand" => RoundMode::HalfExpand,
        "halfTrunc" => RoundMode::HalfTrunc,
        "halfEven" => RoundMode::HalfEven,
        _ => return Err(savvy::Error::new(format!("invalid rounding mode '{x}'"))),
    })
}

/// Options shared by `until()`/`since()`.
#[derive(Clone, Copy)]
pub(crate) struct DiffOpts {
    pub largest: Option<Unit>,
    pub smallest: Unit,
    pub increment: i64,
    pub mode: RoundMode,
}

impl DiffOpts {
    /// Temporal computes `x.since(y)` as the negation of `x.until(y)` rounded
    /// with the negated mode (jiff's `since()` measures from `y` instead,
    /// which differs for calendar units), so `since` uses this mode.
    pub(crate) fn mode_for(&self, since: bool) -> RoundMode {
        if !since {
            return self.mode;
        }
        match self.mode {
            RoundMode::Ceil => RoundMode::Floor,
            RoundMode::Floor => RoundMode::Ceil,
            RoundMode::HalfCeil => RoundMode::HalfFloor,
            RoundMode::HalfFloor => RoundMode::HalfCeil,
            m => m,
        }
    }

    pub(crate) fn new(
        largest: &str,
        smallest: &str,
        increment: f64,
        mode: &str,
    ) -> savvy::Result<Self> {
        Ok(Self {
            largest: parse_unit_auto(largest)?,
            smallest: parse_unit(smallest)?,
            increment: increment_i64(increment)?,
            mode: parse_round_mode(mode)?,
        })
    }
}

pub(crate) fn increment_i64(x: f64) -> savvy::Result<i64> {
    if !x.is_finite() || !(1.0..=1e9).contains(&x) || x.fract() != 0.0 {
        return Err(savvy::Error::new(format!(
            "rounding increment {x} must be a whole number between 1 and 1e9"
        )));
    }
    Ok(x as i64)
}
