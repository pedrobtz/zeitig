//! Temporal's string rules that jiff's parser does not enforce on its own.
//!
//! `prepare()` runs before every RFC 9557 parse. It checks the UTC offset and
//! the calendar annotations and returns the string with the calendar
//! annotations removed: jiff ignores non-critical `[u-ca=...]` annotations but
//! rejects critical ones, while Temporal accepts `[!u-ca=iso8601]`.

use std::borrow::Cow;

/// What the string is parsed as.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Plain dates and plain date-times: only the ISO 8601 calendar is
    /// supported.
    Calendar,
    /// Zoned date-times: the rules of both `Calendar` and `Instant`.
    Zoned,
    /// Plain times: the calendar annotation is ignored.
    Time,
    /// Instants: the calendar annotation is ignored and the UTC offset must
    /// be a whole number of seconds (jiff's offset precision).
    Instant,
}

pub(crate) fn prepare(s: &str, kind: Kind) -> Result<Cow<'_, str>, String> {
    let (main, annotations) = match s.find('[') {
        Some(k) => s.split_at(k),
        None => (s, ""),
    };
    check_offset(main, kind)?;
    if !annotations.contains("u-ca=") {
        return Ok(Cow::Borrowed(s));
    }
    let mut kept = String::from(main);
    let mut calendar: Option<&str> = None;
    let (mut n, mut critical) = (0, false);
    let mut rest = annotations;
    while !rest.is_empty() {
        // Malformed annotations are left for jiff to report.
        let Some(end) = rest.find(']').filter(|_| rest.starts_with('[')) else {
            kept.push_str(rest);
            break;
        };
        let body = &rest[1..end];
        let (crit, body) = match body.strip_prefix('!') {
            Some(b) => (true, b),
            None => (false, body),
        };
        match body.strip_prefix("u-ca=") {
            Some(value) => {
                n += 1;
                critical |= crit;
                calendar.get_or_insert(value);
            }
            None => kept.push_str(&rest[..=end]),
        }
        rest = &rest[end + 1..];
    }
    if n > 1 && critical {
        return Err("a critical calendar annotation must be the only one".to_string());
    }
    if let (Kind::Calendar | Kind::Zoned, Some(cal)) = (kind, calendar) {
        if !cal.eq_ignore_ascii_case("iso8601") {
            return Err(format!(
                "calendar '{cal}' is not supported: only the ISO 8601 calendar ('iso8601') is"
            ));
        }
    }
    Ok(Cow::Owned(kept))
}

/// Temporal limits UTC offsets to less than 24 hours (jiff allows 25:59:59),
/// and an instant's offset may have fractional seconds, which jiff would
/// silently drop.
fn check_offset(main: &str, kind: Kind) -> Result<(), String> {
    let time = match main.find(['T', 't', ' ']) {
        Some(k) => &main[k + 1..],
        None if kind == Kind::Time => main,
        None => return Ok(()),
    };
    let Some(k) = time.find(['+', '-']) else {
        return Ok(());
    };
    let offset = &time[k..];
    let hours: Option<u32> = offset.get(1..3).and_then(|h| h.parse().ok());
    if hours.is_some_and(|h| h >= 24) {
        return Err(format!(
            "UTC offset '{offset}' is out of range: it must be less than 24 hours"
        ));
    }
    if matches!(kind, Kind::Instant | Kind::Zoned) && offset.contains(['.', ',']) {
        return Err(format!(
            "UTC offset '{offset}' has fractional seconds, which are not supported"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendars() {
        let ok = |s, k| prepare(s, k).unwrap().into_owned();
        assert_eq!(ok("2020-01-01[u-ca=iso8601]", Kind::Calendar), "2020-01-01");
        assert_eq!(
            ok("2020-01-01[!u-ca=ISO8601]", Kind::Calendar),
            "2020-01-01"
        );
        assert_eq!(
            ok(
                "2020-01-01T00:00[Europe/Paris][u-ca=iso8601]",
                Kind::Calendar
            ),
            "2020-01-01T00:00[Europe/Paris]"
        );
        assert_eq!(
            ok("2020-01-01[u-ca=iso8601][u-ca=hebrew]", Kind::Calendar),
            "2020-01-01"
        );
        assert_eq!(ok("12:00[u-ca=hebrew]", Kind::Time), "12:00");
        assert!(prepare("2020-01-01[u-ca=hebrew]", Kind::Calendar).is_err());
        assert!(prepare("2020-01-01[u-ca=iso8601][!u-ca=iso8601]", Kind::Calendar).is_err());
        assert_eq!(
            ok("2020-01-01[foo=bar]", Kind::Calendar),
            "2020-01-01[foo=bar]"
        );
    }

    #[test]
    fn offsets() {
        assert!(prepare("2020-01-01T12:00+23:59", Kind::Calendar).is_ok());
        assert!(prepare("2020-01-01T12:00+24:00", Kind::Calendar).is_err());
        assert!(prepare("2020-01-01T12:00-25:00", Kind::Instant).is_err());
        assert!(prepare("12:00:00+24:00", Kind::Time).is_err());
        assert!(prepare("2020-01-01", Kind::Calendar).is_ok());
        assert!(prepare("2020-01-01T12:00+01:00:30.5", Kind::Calendar).is_ok());
        assert!(prepare("2020-01-01T12:00+01:00:30.5", Kind::Instant).is_err());
        assert!(prepare("2020-01-01T12:00:00.5Z", Kind::Instant).is_ok());
        assert!(prepare("2020-01-01T12:00+01:00:30.5[+01:00]", Kind::Zoned).is_err());
    }
}
