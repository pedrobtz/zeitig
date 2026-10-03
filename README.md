
# zudate

<!-- badges: start -->
[![R-CMD-check](https://github.com/pedrobtz/zudate/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/pedrobtz/zudate/actions/workflows/R-CMD-check.yaml)
[![coverage](https://raw.githubusercontent.com/pedrobtz/zudate/main/.github/badges/coverage.svg)](https://github.com/pedrobtz/zudate/actions/workflows/coverage.yaml)
<!-- badges: end -->

zudate brings the [TC39 Temporal](https://tc39.es/proposal-temporal/docs/) date/time model
(`Instant`, `PlainDate`, `PlainTime`, `PlainDateTime`, `ZonedDateTime`, `Duration`) to R as
vectorised classes, with all date/time logic delegated to the Rust crate
[jiff](https://docs.rs/jiff). The package is under active development towards 0.1.0; see
`roadmap.md` for progress.

Building from source needs a Rust toolchain (`cargo` and `rustc` >= 1.81, see
<https://www.rust-lang.org/tools/install>). All Rust dependencies are bundled, so no network
access is needed during installation.

## Installation

You can install the development version of zudate from [GitHub](https://github.com/) with:

``` r
# install.packages("pak")
pak::pak("pedrobtz/zudate")
```

## Example

``` r
library(zudate)
head(available_time_zones())
```

