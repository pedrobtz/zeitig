# zudate

<!-- badges: start -->
[![R-CMD-check](https://github.com/pedrobtz/zudate/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/pedrobtz/zudate/actions/workflows/R-CMD-check.yaml)
[![coverage](https://raw.githubusercontent.com/pedrobtz/zudate/main/.github/badges/coverage.svg)](https://github.com/pedrobtz/zudate/actions/workflows/coverage.yaml)
<!-- badges: end -->

zudate brings the [TC39 Temporal](https://tc39.es/proposal-temporal/docs/) date/time model to R:
`PlainDate`, `PlainTime`, `PlainDateTime`, `Instant`, `ZonedDateTime` and `Duration` as
vectorised R classes with nanosecond precision, RFC 9557 strings, DST-aware arithmetic and
Temporal's rounding and disambiguation rules. All date/time logic is delegated to the Rust crate
[jiff](https://docs.rs/jiff), which is bundled with the package.

## Installation

You can install the development version of zudate from [GitHub](https://github.com/pedrobtz/zudate)
with:

``` r
# install.packages("pak")
pak::pak("pedrobtz/zudate")
```

Building from source needs a Rust toolchain (`cargo` and `rustc` >= 1.81, see
<https://www.rust-lang.org/tools/install>). All Rust dependencies are bundled, so no network access
is needed during installation.

## Example

``` r
library(zudate)

# Calendar arithmetic clamps to the end of the month, as in Temporal
plain_date(2021, 1, 31) + duration(months = 1)
#> <plain_date[1]>
#> [1] 2021-02-28

# Differences, balanced and rounded
temporal_until(plain_date(2006, 8, 24), plain_date(2019, 1, 31), largest_unit = "year")
#> <duration[1]>
#> [1] P12Y5M7D

# Time zones: one calendar day later is still noon, but only 23 hours elapsed
z <- zoned_date_time("2020-03-07T12:00-05:00[America/New_York]")
z + duration(days = 1)
#> <zoned_date_time[1]>
#> [1] 2020-03-08T12:00:00-04:00[America/New_York]
(z + duration(days = 1)) - z
#> <duration[1]>
#> [1] PT23H

# Local times in a DST gap are resolved explicitly
zoned_date_time(2019, 3, 10, 2, 30, time_zone = "America/New_York", disambiguation = "earlier")
#> <zoned_date_time[1]>
#> [1] 2019-03-10T01:30:00-05:00[America/New_York]

# Nanosecond precision, exactly
x <- instant("2019-03-30T00:45:00.123456789Z")
epoch_nanoseconds(x)
#> [1] "1553906700123456789"
```

See `vignette("zudate")` for an overview and `vignette("time-zones")` for time zone handling.
