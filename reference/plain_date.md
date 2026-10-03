# Plain dates

A `zeitig_plain_date` is a calendar date without a time or time zone,
the equivalent of
[`Temporal.PlainDate`](https://tc39.es/proposal-temporal/docs/plaindate.html).
Only the ISO 8601 calendar is supported.

## Usage

``` r
plain_date(year, month, day, ..., overflow = c("constrain", "reject"))

is_plain_date(x)
```

## Arguments

- year, month, day:

  Integer-valued numbers. Fractions are truncated towards zero. `year`
  may instead be a character vector to parse.

- ...:

  These dots are for future extensions and must be empty.

- overflow:

  How to handle out-of-range values: `"constrain"` (the default) clamps
  `month` to 12 and `day` to the length of the month, `"reject"` raises
  an error. Zero or negative months and days are always an error.

- x:

  An object to test or convert.

## Value

A `zeitig_plain_date` vector.

## Details

`plain_date()` builds dates from their components (recycled to a common
length) or, when given a single character vector, parses [RFC
9557](https://www.rfc-editor.org/rfc/rfc9557) / ISO 8601 strings.
Strings that also contain a time, an offset or a time zone annotation
are accepted and those parts are ignored, but a UTC designator (`Z`) is
an error, as in Temporal.

## Examples

``` r
plain_date(2006, 8, 24)
#> <plain_date[1]>
#> [1] 2006-08-24
plain_date(2021, 2, 31) # constrained to 2021-02-28
#> <plain_date[1]>
#> [1] 2021-02-28
try(plain_date(2021, 2, 31, overflow = "reject"))
#> Error in plain_date(2021, 2, 31, overflow = "reject") : 
#>   parameter 'day' with value 31 is not in the required range of 1..=28 (element 1)
plain_date(c("2006-08-24", "2019-11-18T15:23:30+01:00[Europe/Paris]", NA))
#> <plain_date[3]>
#> [1] 2006-08-24 2019-11-18 <NA>      
```
