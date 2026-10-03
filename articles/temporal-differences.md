# Differences from Temporal

``` r

library(zeitig)
#> 
#> Attaching package: 'zeitig'
#> The following object is masked from 'package:stats':
#> 
#>     offset
```

zeitig follows the [Temporal](https://tc39.es/proposal-temporal/docs/)
specification, but all date and time logic comes from the Rust crate
[jiff](https://docs.rs/jiff), which is modeled on Temporal without
claiming to implement it exactly. Code ported from JavaScript works as
expected in most cases. This article lists the differences you are most
likely to notice.

## How closely zeitig follows Temporal

The package’s tests run about 6,300 operations through both zeitig and
the reference Temporal implementation and compare the results. The
operations cover parsing, arithmetic, differences and rounding with
every unit and rounding mode, formatting options, and time zone
transitions, and include a selection of cases from Temporal’s official
test suite (test262). 98.8% of the cases give the same result. The
remaining 1.2% are the differences described below, mostly dates beyond
the year 9999. No other case may differ, or the tests fail.

zeitig implements 160 of the 216 methods and properties of Temporal’s
classes (74%). Most of the rest belong to `PlainYearMonth`,
`PlainMonthDay`, calendars and `toLocaleString()`.

## Only the ISO 8601 calendar

Temporal supports many calendars (Hebrew, Islamic, Japanese, …); jiff,
and therefore zeitig, supports only the ISO 8601 (proleptic Gregorian)
calendar. A string with another calendar is an error rather than being
read as ISO 8601:

``` r

plain_date("2024-03-15[u-ca=iso8601]")
#> <plain_date[1]>
#> [1] 2024-03-15
plain_date("2024-03-15[u-ca=hebrew]")
#> Error in `plain_date()`:
#> ! calendar 'hebrew' is not supported: only the ISO 8601 calendar ('iso8601') is (element 1)
```

Some other parts of Temporal are missing too:

- There are no `PlainYearMonth` or `PlainMonthDay` types. Use a
  [`plain_date()`](https://pedrobtz.github.io/zeitig/reference/plain_date.md)
  and ignore the fields you do not need.
- There is no `toLocaleString()`. Use
  [`temporal_strftime()`](https://pedrobtz.github.io/zeitig/reference/temporal_strftime.md)
  for custom formats:

``` r

temporal_strftime(plain_date(2024, 3, 15), "%A %d %B %Y")
#> [1] "Friday 15 March 2024"
```

## A smaller range of dates

Temporal allows instants about 273,790 years either side of 1970. zeitig
only covers the years -9999 to 9999 (instants stop a day or two short of
either end, so that any UTC offset can be applied), and a duration can
be at most 19,998 years:

``` r

plain_date(9999, 12, 31)
#> <plain_date[1]>
#> [1] 9999-12-31
plain_date(10000, 1, 1)
#> Error in `plain_date()`:
#> ! parameter 'year' with value 10000 is not in the required range of -9999..=9999 (element 1)
```

## Comparing durations

Temporal durations have no `==` or `<`. In zeitig, `==` compares the
fields of two durations, so one hour is not `==` to 60 minutes. To
compare their lengths, use
[`duration_compare()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md):

``` r

duration(hours = 1) == duration(minutes = 60)
#> [1] FALSE
duration_compare(duration(hours = 1), duration(minutes = 60))
#> [1] 0
```

`<`, [`sort()`](https://rdrr.io/r/base/sort.html) and
[`duration_total()`](https://pedrobtz.github.io/zeitig/reference/duration_total.md)
compare by length. Without a `relative_to` date, they treat days as 24
hours and weeks as 7 days. (Temporal requires `relativeTo` for weeks.)
Durations in months or years have no fixed length, so they need a
`relative_to` date:

``` r

sort(c(duration(days = 2), duration(hours = 36)))
#> <duration[2]>
#> [1] PT36H P2D
duration_total(duration(weeks = 2), "hour")
#> [1] 336
duration(months = 1) < duration(days = 31)
#> Error in `vec_proxy_compare()`:
#> ! could not compute normalized relative span when all days are assumed to be 24 hours: using unit 'month' in span or configuration requires that a relative reference time be given (`jiff::SpanRelativeTo::days_are_24_hours()` was given but this only permits using days and weeks without a relative reference time) (element 1)
duration_compare(duration(months = 1), duration(days = 31), relative_to = plain_date(2021, 2, 1))
#> [1] -1
```

In a time zone with daylight saving time, a day is not always 24 hours.
Pass a
[`zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md)
as `relative_to` to account for that:

``` r

z <- zoned_date_time("2020-03-07T12:00-05:00[America/New_York]")
duration_total(duration(days = 1), "hour", relative_to = z)
#> [1] 23
```

## Comparing zoned date-times

`==`, `<` and [`sort()`](https://rdrr.io/r/base/sort.html) on zoned
date-times compare only the exact time. To also compare the time zone,
as Temporal’s `equals()` does, use
[`temporal_equals()`](https://pedrobtz.github.io/zeitig/reference/temporal_compare.md):

``` r

utc <- zoned_date_time("2020-01-01T12:00+00:00[UTC]")
london <- with_time_zone(utc, "Europe/London")
utc == london
#> [1] TRUE
temporal_equals(utc, london)
#> [1] FALSE
```

Like Temporal,
[`temporal_equals()`](https://pedrobtz.github.io/zeitig/reference/temporal_compare.md)
treats `UTC`, `Etc/UTC`, `Etc/GMT` and `GMT` as the same zone, but it
does not resolve other alternative names, such as `Asia/Calcutta` for
`Asia/Kolkata`.

## Nanoseconds since the epoch

Temporal returns `epochNanoseconds` as a `BigInt`. R has no 64-bit
integer type, and a double cannot hold nanoseconds exactly, so
[`epoch_nanoseconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
returns a string.
[`instant_from_epoch()`](https://pedrobtz.github.io/zeitig/reference/instant.md)
accepts the same string:

``` r

x <- instant("2019-03-30T00:45:00.123456789Z")
epoch_nanoseconds(x)
#> [1] "1553906700123456789"
instant_from_epoch(nanoseconds = epoch_nanoseconds(x)) == x
#> [1] TRUE
```

[`epoch_seconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
and
[`epoch_milliseconds()`](https://pedrobtz.github.io/zeitig/reference/epoch_seconds.md)
return doubles, rounded down to whole units.

## UTC offsets with seconds

Before about 1900, many places used local mean time, with UTC offsets
that are not whole minutes. Temporal prints such offsets rounded to the
minute; zeitig prints the seconds, so that the string reads back as the
same value. A UTC offset with fractional seconds, which Temporal
accepts, is an error in zeitig:

``` r

format(instant("1900-01-01T12:00Z"), time_zone = "Europe/Dublin")
#> [1] "1900-01-01T11:34:39-00:25:21"
instant("2020-01-01T12:00+01:00:30.5")
#> Error in `instant()`:
#> ! UTC offset '+01:00:30.5' has fractional seconds, which are not supported (element 1)
```

## R conveniences

These do not change any results, but they have no equivalent in
Temporal:

- Every function is vectorised, and each element of a zoned date-time
  can have its own time zone.
- `+` and `-` with a duration call
  [`temporal_add()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md)
  and
  [`temporal_subtract()`](https://pedrobtz.github.io/zeitig/reference/temporal_add.md).
  `x - y` for two values of the same type is `temporal_since(x, y)`:

``` r

plain_date(2021, 1, 31) - plain_date(2020, 12, 1)
#> <duration[1]>
#> [1] P61D
```
