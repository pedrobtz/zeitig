//! Time zone database access and identifier handling.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use jiff::tz::{Disambiguation, OffsetConflict, TimeZone, TimeZoneDatabase};
use savvy::{savvy, OwnedStringSexp, StringSexp};

use crate::cols::{elt_error, is_na_str};

/// The database used for every lookup. `ZEITIG_TZDIR` (read once per
/// session) points at a zoneinfo directory to use instead of jiff's default
/// (the system database, honouring `TZDIR`; the bundled copy on Windows).
/// An override that does not hold a readable database is an error on every
/// lookup rather than a silent fallback to the default database.
pub(crate) fn db() -> Result<&'static TimeZoneDatabase, String> {
    static DB: OnceLock<Result<TimeZoneDatabase, String>> = OnceLock::new();
    DB.get_or_init(|| match std::env::var_os("ZEITIG_TZDIR") {
        Some(dir) if !dir.is_empty() => {
            let shown = std::path::Path::new(&dir).display().to_string();
            match TimeZoneDatabase::from_dir(&dir) {
                Ok(db) if !db.is_definitively_empty() => Ok(db),
                Ok(_) => Err(format!(
                    "ZEITIG_TZDIR ('{shown}') does not contain a time zone database"
                )),
                Err(e) => Err(format!(
                    "ZEITIG_TZDIR ('{shown}') is not a readable time zone database: {e}"
                )),
            }
        }
        _ => Ok(jiff::tz::db().clone()),
    })
    .as_ref()
    .map_err(|e| e.clone())
}

/// Sorted IANA time zone identifiers known to the time zone database.
pub(crate) fn time_zone_names() -> Result<Vec<String>, String> {
    let mut names: Vec<String> = db()?.available().map(|n| n.to_string()).collect();
    names.sort_unstable();
    names.dedup();
    Ok(names)
}

/// Resolves a Temporal time zone identifier: an IANA name (case-insensitive)
/// or a fixed offset `±HH:MM`. Returns the zone and its canonical spelling.
pub(crate) fn resolve_time_zone(id: &str) -> Result<(TimeZone, String), String> {
    if id.starts_with('+') || id.starts_with('-') {
        let tz = jiff::fmt::temporal::DateTimeParser::new()
            .parse_time_zone(id)
            .map_err(|e| format!("invalid time zone offset '{id}': {e}"))?;
        let offset = tz
            .to_fixed_offset()
            .map_err(|e| format!("invalid time zone offset '{id}': {e}"))?;
        if offset.seconds() % 60 != 0 {
            return Err(format!(
                "invalid time zone offset '{id}': sub-minute offsets are not allowed"
            ));
        }
        let canonical = format_offset(offset.seconds());
        if offset.seconds() == 0 {
            // jiff represents a zero fixed offset as its UTC zone; Temporal
            // keeps `+00:00` apart from `UTC`.
            return Ok((zero_offset_zone()?, canonical));
        }
        return Ok((tz, canonical));
    }
    if id.is_empty() || id.contains(',') {
        return Err(format!("invalid time zone identifier '{id}'"));
    }
    let tz = db()?
        .get(id)
        .map_err(|_| format!("unknown time zone identifier '{id}'"))?;
    let canonical = tz.iana_name().unwrap_or(id).to_string();
    Ok((tz, canonical))
}

/// A fixed `+00:00` zone that is not jiff's `TimeZone::UTC`: a TZif zone
/// named `+00:00`, so `iana_name()` (and hence `time_zone_id()` and jiff's
/// `Display`) gives `+00:00`.
fn zero_offset_zone() -> Result<TimeZone, String> {
    static ZONE: OnceLock<Result<TimeZone, String>> = OnceLock::new();
    ZONE.get_or_init(|| {
        // RFC 8536 TZif v2: no transitions, one local time type (offset 0,
        // abbreviation "+00"), and the POSIX footer `<+00>0`.
        let header: [u8; 44] = {
            let mut h = [0u8; 44];
            h[..5].copy_from_slice(b"TZif2");
            // ttisutcnt, ttisstdcnt, leapcnt, timecnt = 0; typecnt = 1; charcnt = 4
            h[39] = 1;
            h[43] = 4;
            h
        };
        let data: [u8; 10] = [0, 0, 0, 0, 0, 0, b'+', b'0', b'0', 0];
        let mut tzif = Vec::with_capacity(2 * (44 + 10) + 8);
        for _ in 0..2 {
            tzif.extend_from_slice(&header);
            tzif.extend_from_slice(&data);
        }
        tzif.extend_from_slice(b"\n<+00>0\n");
        TimeZone::tzif("+00:00", &tzif).map_err(|e| format!("internal error: {e}"))
    })
    .clone()
}

/// The Temporal identifier of a zone: its IANA name or its fixed offset.
/// Zones that are neither (POSIX TZ strings) are not Temporal time zones and
/// are an error, never silently reported as UTC.
pub(crate) fn time_zone_id(tz: &TimeZone) -> Result<String, String> {
    if let Some(name) = tz.iana_name() {
        return Ok(name.to_string());
    }
    match tz.to_fixed_offset() {
        Ok(offset) => Ok(format_offset(offset.seconds())),
        Err(_) => Err(
            "unsupported time zone: only IANA identifiers and fixed UTC offsets are allowed"
                .to_string(),
        ),
    }
}

/// ECMA-402 resolves "Etc/UTC", "Etc/GMT" and "GMT" to "UTC"; no other links
/// are resolved (design.md section 9). Keep in sync with
/// `time_zone_primary()` in `R/compare.R`.
pub(crate) fn time_zone_primary(id: &str) -> &str {
    match id {
        "Etc/UTC" | "Etc/GMT" | "GMT" => "UTC",
        _ => id,
    }
}

/// Temporal `TimeZoneEquals` for two resolved identifiers.
pub(crate) fn time_zones_equal(a: &str, b: &str) -> bool {
    a == b || time_zone_primary(a) == time_zone_primary(b)
}

/// `+HH:MM`, or `+HH:MM:SS` when the offset has seconds (Temporal's
/// `offset` string format).
pub(crate) fn format_offset(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let s = seconds.unsigned_abs();
    let (h, m, sec) = (s / 3600, s / 60 % 60, s % 60);
    if sec == 0 {
        format!("{sign}{h:02}:{m:02}")
    } else {
        format!("{sign}{h:02}:{m:02}:{sec:02}")
    }
}

/// A resolved time zone and its canonical identifier. The identifier is
/// shared (`Rc`) so writing it into every output element allocates nothing.
#[derive(Clone)]
pub(crate) struct Resolved {
    pub tz: TimeZone,
    pub id: Rc<str>,
}

/// Per-call cache so a column with one distinct zone pays one lookup. Hits
/// allocate nothing: the most recent identifier is compared first (columns
/// usually repeat one zone), then the map is probed by `&str`.
#[derive(Default)]
pub(crate) struct TzCache {
    keys: Vec<String>,
    entries: Vec<Result<Resolved, String>>,
    index: HashMap<String, usize>,
    last: Option<usize>,
}

impl TzCache {
    pub(crate) fn get(&mut self, i: usize, id: &str) -> savvy::Result<&Resolved> {
        let k = match self.last {
            Some(k) if self.keys[k] == id => k,
            _ => {
                let k = match self.index.get(id) {
                    Some(&k) => k,
                    None => {
                        let k = self.entries.len();
                        let entry = resolve_time_zone(id).map(|(tz, id)| Resolved {
                            tz,
                            id: Rc::from(id),
                        });
                        self.entries.push(entry);
                        self.keys.push(id.to_string());
                        self.index.insert(id.to_string(), k);
                        k
                    }
                };
                self.last = Some(k);
                k
            }
        };
        self.entries[k].as_ref().map_err(|e| elt_error(i, e))
    }
}

pub(crate) fn parse_disambiguation(x: &str) -> savvy::Result<Disambiguation> {
    Ok(match x {
        "compatible" => Disambiguation::Compatible,
        "earlier" => Disambiguation::Earlier,
        "later" => Disambiguation::Later,
        "reject" => Disambiguation::Reject,
        _ => return Err(savvy::Error::new(format!("invalid disambiguation '{x}'"))),
    })
}

pub(crate) fn parse_offset_conflict(x: &str) -> savvy::Result<OffsetConflict> {
    Ok(match x {
        "use" => OffsetConflict::AlwaysOffset,
        "prefer" => OffsetConflict::PreferOffset,
        "ignore" => OffsetConflict::AlwaysTimeZone,
        "reject" => OffsetConflict::Reject,
        _ => return Err(savvy::Error::new(format!("invalid offset option '{x}'"))),
    })
}

#[savvy]
fn rs_available_time_zones() -> savvy::Result<savvy::Sexp> {
    let names = time_zone_names().map_err(savvy::Error::new)?;
    let out = OwnedStringSexp::try_from_iter(names.iter())?;
    Ok(out.into())
}

// Validates time zone identifiers and returns their canonical spelling.
#[savvy]
fn rs_time_zone_canonical(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut cache = TzCache::default();
    let mut out = OwnedStringSexp::new(x.len())?;
    for (i, id) in x.iter().enumerate() {
        if is_na_str(id) {
            out.set_na(i)?;
        } else {
            out.set_elt(i, &cache.get(i, id)?.id)?;
        }
    }
    Ok(out.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_sorted_and_unique() {
        let names = time_zone_names().unwrap();
        assert!(names.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn identifiers() {
        assert_eq!(resolve_time_zone("europe/paris").unwrap().1, "Europe/Paris");
        assert_eq!(resolve_time_zone("UTC").unwrap().1, "UTC");
        assert_eq!(resolve_time_zone("+05:30").unwrap().1, "+05:30");
        let (zero, id) = resolve_time_zone("+00:00").unwrap();
        assert_eq!(id, "+00:00");
        assert_eq!(time_zone_id(&zero).unwrap(), "+00:00");
        assert_eq!(zero.to_offset(jiff::Timestamp::UNIX_EPOCH).seconds(), 0);
        assert_ne!(zero, TimeZone::UTC);
        assert_eq!(resolve_time_zone("-0800").unwrap().1, "-08:00");
        assert!(resolve_time_zone("+05:30:15").is_err());
        assert!(resolve_time_zone("Mars/Olympus").is_err());
        assert!(resolve_time_zone("EST5EDT,M3.2.0,M11.1.0").is_err());
        assert!(resolve_time_zone("").is_err());
        assert_eq!(format_offset(-(5 * 3600 + 30 * 60)), "-05:30");
        assert_eq!(format_offset(-1258), "-00:20:58");
        let posix = TimeZone::posix("EST5EDT,M3.2.0,M11.1.0").unwrap();
        assert!(time_zone_id(&posix).is_err());
    }

    #[test]
    fn primary_zones() {
        assert!(time_zones_equal("UTC", "Etc/UTC"));
        assert!(time_zones_equal("GMT", "Etc/GMT"));
        assert!(!time_zones_equal("UTC", "+00:00"));
        assert!(!time_zones_equal("Europe/London", "UTC"));
    }

    #[test]
    fn cache_reuses_entries() {
        let mut cache = TzCache::default();
        let a = cache.get(0, "Europe/Paris").unwrap().id.clone();
        let b = cache.get(1, "europe/paris").unwrap().id.clone();
        let c = cache.get(2, "Europe/Paris").unwrap().id.clone();
        assert_eq!(&*a, "Europe/Paris");
        assert_eq!(&*b, "Europe/Paris");
        assert!(Rc::ptr_eq(&a, &c));
        assert!(cache.get(3, "Mars/Base").is_err());
        assert!(cache.get(4, "Mars/Base").is_err());
        assert_eq!(cache.entries.len(), 3);
    }
}
