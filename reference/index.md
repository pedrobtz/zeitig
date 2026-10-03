# Package index

## Plain dates and times

Calendar dates and wall-clock times without a time zone.

- [`plain_date()`](https://pedrobtz.github.io/zeitig/reference/plain_date.md)
  [`is_plain_date()`](https://pedrobtz.github.io/zeitig/reference/plain_date.md)
  : Plain dates
- [`plain_time()`](https://pedrobtz.github.io/zeitig/reference/plain_time.md)
  [`is_plain_time()`](https://pedrobtz.github.io/zeitig/reference/plain_time.md)
  : Plain times
- [`plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/plain_date_time.md)
  [`is_plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/plain_date_time.md)
  : Plain date-times

## Exact time

Instants and zoned date-times.

- [`instant()`](https://pedrobtz.github.io/zeitig/reference/instant.md)
  [`instant_from_epoch()`](https://pedrobtz.github.io/zeitig/reference/instant.md)
  [`is_instant()`](https://pedrobtz.github.io/zeitig/reference/instant.md)
  : Instants
- [`epoch_seconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
  [`epoch_milliseconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
  [`epoch_nanoseconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
  : Epoch time
- [`zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md)
  [`is_zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md)
  : Zoned date-times
- [`time_zone()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`offset()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`offset_nanoseconds()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`hours_in_day()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`start_of_day()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`time_zone_transition()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  [`with_time_zone()`](https://pedrobtz.github.io/zeitig/reference/time_zone.md)
  : Time zone information
- [`available_time_zones()`](https://pedrobtz.github.io/zeitig/reference/available_time_zones.md)
  : Available time zones

## Durations

- [`duration()`](https://pedrobtz.github.io/zeitig/reference/duration.md)
  [`is_duration()`](https://pedrobtz.github.io/zeitig/reference/duration.md)
  : Durations
- [`duration_total()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
  [`duration_compare()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
  [`duration_blank()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
  : Duration helpers
- [`as_duration()`](https://pedrobtz.github.io/zeitig/reference/as_duration.md)
  [`as_difftime()`](https://pedrobtz.github.io/zeitig/reference/as_duration.md)
  : Coerce to and from durations

## Fields and modification

- [`year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`month()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`day()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`hour()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`minute()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`second()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`millisecond()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`microsecond()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`nanosecond()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`day_of_week()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`day_of_year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`week_of_year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`year_of_week()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`days_in_week()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`days_in_month()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`days_in_year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`months_in_year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`in_leap_year()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  [`temporal_fields()`](https://pedrobtz.github.io/zeitig/reference/temporal-fields.md)
  : Date and time fields
- [`temporal_with()`](https://pedrobtz.github.io/zeitig/reference/temporal_with.md)
  [`with_plain_time()`](https://pedrobtz.github.io/zeitig/reference/temporal_with.md)
  [`with_plain_date()`](https://pedrobtz.github.io/zeitig/reference/temporal_with.md)
  : Replace fields

## Arithmetic, rounding and comparison

- [`temporal_add()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md)
  [`temporal_subtract()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md)
  : Add or subtract durations
- [`temporal_until()`](https://pedrobtz.github.io/zeitig/reference/temporal_until.md)
  [`temporal_since()`](https://pedrobtz.github.io/zeitig/reference/temporal_until.md)
  : Difference between two values
- [`temporal_round()`](https://pedrobtz.github.io/zeitig/reference/temporal_round.md)
  : Round values and durations
- [`temporal_compare()`](https://pedrobtz.github.io/zeitig/reference/temporal_compare.md)
  [`temporal_equals()`](https://pedrobtz.github.io/zeitig/reference/temporal_compare.md)
  : Compare Temporal objects

## Conversion, parsing and formatting

- [`to_plain_date()`](https://pedrobtz.github.io/zeitig/reference/temporal-conversions.md)
  [`to_plain_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-conversions.md)
  [`to_plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-conversions.md)
  [`to_instant()`](https://pedrobtz.github.io/zeitig/reference/temporal-conversions.md)
  [`to_zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-conversions.md)
  : Convert between Temporal types
- [`as_plain_date()`](https://pedrobtz.github.io/zeitig/reference/temporal-coercion.md)
  [`as_plain_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-coercion.md)
  [`as_plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-coercion.md)
  [`as_instant()`](https://pedrobtz.github.io/zeitig/reference/temporal-coercion.md)
  [`as_zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/temporal-coercion.md)
  : Coerce to Temporal types
- [`format(`*`<zeitig_plain_date>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-format.md)
  [`format(`*`<zeitig_plain_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-format.md)
  [`format(`*`<zeitig_plain_date_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-format.md)
  [`format(`*`<zeitig_instant>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-format.md)
  [`format(`*`<zeitig_zoned_date_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-format.md)
  : Format Temporal objects as strings
- [`temporal_strftime()`](https://pedrobtz.github.io/zeitig/reference/temporal_strftime.md)
  [`temporal_strptime()`](https://pedrobtz.github.io/zeitig/reference/temporal_strftime.md)
  : strftime-style formatting and parsing
- [`summary(`*`<zeitig_plain_date>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  [`summary(`*`<zeitig_plain_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  [`summary(`*`<zeitig_plain_date_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  [`summary(`*`<zeitig_instant>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  [`summary(`*`<zeitig_zoned_date_time>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  [`summary(`*`<zeitig_duration>`*`)`](https://pedrobtz.github.io/zeitig/reference/temporal-summary.md)
  : Summaries of Temporal vectors

## Current time

- [`now_instant()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  [`now_zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  [`now_plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  [`now_plain_date()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  [`now_plain_time()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  [`now_time_zone()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
  : The current time

## Package

- [`zeitig`](https://pedrobtz.github.io/zeitig/reference/zeitig-package.md)
  [`zeitig-package`](https://pedrobtz.github.io/zeitig/reference/zeitig-package.md)
  : zeitig: 'Temporal' Date and Time Types Backed by 'jiff'
