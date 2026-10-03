# Date and time fields

Accessors for the fields of Temporal objects, the equivalent of the
getters such as `.year`, `.dayOfWeek` or `.inLeapYear`. All are
vectorised and return `NA` for missing elements.

## Usage

``` r
year(x)

month(x)

day(x)

hour(x)

minute(x)

second(x)

millisecond(x)

microsecond(x)

nanosecond(x)

day_of_week(x)

day_of_year(x)

week_of_year(x)

year_of_week(x)

days_in_week(x)

days_in_month(x)

days_in_year(x)

months_in_year(x)

in_leap_year(x)

temporal_fields(x)
```

## Arguments

- x:

  A Temporal object: date fields need a plain date or plain date-time,
  time fields a plain time or plain date-time; `temporal_fields()` also
  accepts durations.

## Value

An integer vector (`in_leap_year()`: logical; `temporal_fields()`: a
data frame).

## Details

- `year()`, `month()`, `day()`: the calendar date (ISO 8601 calendar).

- `hour()`, `minute()`, `second()`, `millisecond()`, `microsecond()`,
  `nanosecond()`: the wall-clock time; `second()` is the whole second
  and the sub-second part is split into the three remaining fields, each
  in `0:999`.

- `day_of_week()`: 1 (Monday) to 7 (Sunday).

- `day_of_year()`: 1 to 366.

- `week_of_year()`, `year_of_week()`: the ISO 8601 week number and the
  year that week belongs to (which differs from `year()` around New
  Year).

- `days_in_week()`, `days_in_month()`, `days_in_year()`,
  `months_in_year()`, `in_leap_year()`.

`temporal_fields()` returns all the component fields as a data frame;
for durations these are the ten fields from `years` to `nanoseconds`.

## Examples

``` r
x <- plain_date_time("2021-01-03T15:23:30.123456789")
year(x)
#> [1] 2021
day_of_week(x)
#> [1] 7
week_of_year(x)
#> [1] 53
year_of_week(x)
#> [1] 2020
c(millisecond(x), microsecond(x), nanosecond(x))
#> [1] 123 456 789
temporal_fields(x)
#>   year month day hour minute second millisecond microsecond nanosecond
#> 1 2021     1   3   15     23     30         123         456        789
```
