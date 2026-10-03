use savvy::{savvy, OwnedStringSexp};

/// Sorted IANA time zone identifiers known to the time zone database.
pub(crate) fn time_zone_names() -> Vec<String> {
    let mut names: Vec<String> = jiff::tz::db().available().map(|n| n.to_string()).collect();
    names.sort_unstable();
    names.dedup();
    names
}

#[savvy]
fn rs_available_time_zones() -> savvy::Result<savvy::Sexp> {
    let names = time_zone_names();
    let out = OwnedStringSexp::try_from_iter(names.iter())?;
    Ok(out.into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_are_sorted_and_unique() {
        let names = super::time_zone_names();
        assert!(names.windows(2).all(|w| w[0] < w[1]));
    }
}
