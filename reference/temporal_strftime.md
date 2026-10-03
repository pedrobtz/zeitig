# strftime-style formatting and parsing

`temporal_strftime()` formats Temporal objects and `temporal_strptime()`
parses strings with `strftime`-style format strings, as implemented by
[`jiff::fmt::strtime`](https://docs.rs/jiff/latest/jiff/fmt/strtime/).
Common directives: `%Y` year, `%m` month, `%d` day, `%H` hour, `%M`
minute, `%S` second, `%f` fractional seconds, `%A`/`%a` weekday name,
`%B`/`%b` month name, `%j` day of year, `%z` UTC offset, `%Z` time zone
abbreviation, `%Q` IANA time zone name.

## Usage

``` r
temporal_strftime(x, format)

temporal_strptime(
  x,
  format,
  class = c("plain_date", "plain_time", "plain_date_time", "instant", "zoned_date_time")
)
```

## Arguments

- x:

  A Temporal object (`temporal_strftime()`) or a character vector
  (`temporal_strptime()`).

- format:

  Format strings, recycled against `x`.

- class:

  The class to parse into.

## Value

A character vector (`temporal_strftime()`) or a vector of the requested
class (`temporal_strptime()`).

## Details

Formatting fails for directives the value cannot supply (e.g. `%H` for a
plain date, `%z` for a plain date-time). Parsing to an instant needs an
offset (`%z`), and to a zoned date-time an offset or time zone name
(`%z`, `%Q`).

## Examples

``` r
temporal_strftime(plain_date(2024, 7, 15), "%A, %B %d, %Y")
#> [1] "Monday, July 15, 2024"
z <- zoned_date_time("2024-07-15T16:24:59-04:00[America/New_York]")
temporal_strftime(z, "%H:%M %Z (%z)")
#> [1] "16:24 EDT (-0400)"
temporal_strptime("15/07/2024", "%d/%m/%Y", "plain_date")
#> <plain_date[1]>
#> [1] 2024-07-15
temporal_strptime("2024-07-15 16:24 -0400", "%Y-%m-%d %H:%M %z", "instant")
#> <instant[1]>
#> [1] 2024-07-15T20:24:00Z
```
