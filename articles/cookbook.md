# Cookbook: everyday tasks with Temporal types

This article collects short recipes for the date and time tasks that
come up most often, in the spirit of the [Temporal
cookbook](https://tc39.es/proposal-temporal/docs/cookbook.html). Each
recipe uses the type that matches the question being asked; the table
below is the quick guide.

| If you have… | use |
|----|----|
| a calendar date (birthday, due date, holiday) | [`plain_date()`](https://pedrobtz.github.io/zeitig/reference/plain_date.md) |
| a wall-clock time (opening hours, alarm) | [`plain_time()`](https://pedrobtz.github.io/zeitig/reference/plain_time.md) |
| a date and time with no place (a form input) | [`plain_date_time()`](https://pedrobtz.github.io/zeitig/reference/plain_date_time.md) |
| an exact moment (log entry, event timestamp) | [`instant()`](https://pedrobtz.github.io/zeitig/reference/instant.md) |
| an exact moment *and* where it happened | [`zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md) |
| a length of time (“3 months”, “90 minutes”) | [`duration()`](https://pedrobtz.github.io/zeitig/reference/duration.md) |

``` r

library(zeitig)
#> 
#> Attaching package: 'zeitig'
#> The following object is masked from 'package:stats':
#> 
#>     offset
```

All functions are vectorised, so every recipe works the same on one
value or on a whole column.

## Getting the current date and time

``` r

now_instant()                          # exact time, no time zone
#> <instant[1]>
#> [1] 2026-10-03T21:44:48.201985644Z
now_zoned_date_time("Europe/Berlin")   # exact time seen in a time zone
#> <zoned_date_time[1]>
#> [1] 2026-10-03T23:44:48.204419964+02:00[Europe/Berlin]
now_plain_date("Asia/Tokyo")           # today's date in Tokyo
#> <plain_date[1]>
#> [1] 2026-10-04
now_plain_time()                       # wall-clock time in the session time zone
#> <plain_time[1]>
#> [1] 21:44:48.207810553
now_time_zone()
#> [1] "UTC"
```

## Parsing strings

Every constructor parses ISO 8601 / RFC 9557 strings. Invalid input is
an error that says which element failed; `NA` stays `NA`.

``` r

plain_date(c("2024-02-29", "2024-12-25", NA))
#> <plain_date[3]>
#> [1] 2024-02-29 2024-12-25 <NA>
plain_time("08:30")
#> <plain_time[1]>
#> [1] 08:30:00
instant("2024-03-10T07:15:00Z")
#> <instant[1]>
#> [1] 2024-03-10T07:15:00Z
zoned_date_time("2024-03-10T08:15:00+01:00[Europe/Berlin]")
#> <zoned_date_time[1]>
#> [1] 2024-03-10T08:15:00+01:00[Europe/Berlin]
duration("P1DT12H")
#> <duration[1]>
#> [1] P1DT12H
plain_date(c("2024-02-29", "2023-02-29"))
#> Error in `plain_date()`:
#> ! parsed date is not valid: parameter 'day' for `2023-02` is invalid, must be in range `1..=28` (element 2)
```

For other layouts, use strftime-style patterns:

``` r

temporal_strptime(c("25/12/2024", "01/01/2025"), "%d/%m/%Y", "plain_date")
#> <plain_date[2]>
#> [1] 2024-12-25 2025-01-01
temporal_strptime("Mar 10 2024 3:45 PM", "%b %d %Y %I:%M %p", "plain_date_time")
#> <plain_date_time[1]>
#> [1] 2024-03-10T15:45:00
```

## Building values from parts

``` r

plain_date(2024, 1:12, 1)                  # first day of every month
#> <plain_date[12]>
#>  [1] 2024-01-01 2024-02-01 2024-03-01 2024-04-01 2024-05-01 2024-06-01
#>  [7] 2024-07-01 2024-08-01 2024-09-01 2024-10-01 2024-11-01 2024-12-01
plain_time(9, c(0, 30))                    # 09:00 and 09:30
#> <plain_time[2]>
#> [1] 09:00:00 09:30:00
plain_date_time(2024, 6, 1, hour = 18)
#> <plain_date_time[1]>
#> [1] 2024-06-01T18:00:00
zoned_date_time(2024, 6, 1, 18, time_zone = "America/New_York")
#> <zoned_date_time[1]>
#> [1] 2024-06-01T18:00:00-04:00[America/New_York]
duration(hours = 1, minutes = 30)
#> <duration[1]>
#> [1] PT1H30M
```

## Taking dates and times apart

``` r

d <- plain_date_time("2024-07-04T16:45:30")
year(d)
#> [1] 2024
month(d)
#> [1] 7
day_of_week(d)   # 1 = Monday ... 7 = Sunday
#> [1] 4
week_of_year(d)
#> [1] 27
hour(d)
#> [1] 16
temporal_fields(d)
#>   year month day hour minute second millisecond microsecond nanosecond
#> 1 2024     7   4   16     45     30           0           0          0
```

## Date arithmetic

Adding months or years keeps the day of the month when it can and clamps
to the end of the month when it cannot:

``` r

d <- plain_date(2024, 1, 31)
d + duration(months = 1:3)
#> <plain_date[3]>
#> [1] 2024-02-29 2024-03-31 2024-04-30
d + duration(years = 1, days = 10)
#> <plain_date[1]>
#> [1] 2025-02-10
d - duration(weeks = 2)
#> <plain_date[1]>
#> [1] 2024-01-17
```

If clamping should be an error instead, ask for it:

``` r

temporal_add(d, duration(months = 1), overflow = "reject")
#> Error in `temporal_add()`:
#> ! day 31 is out of range for 2024-02 (overflow = "reject") (element 1)
```

## Days between two dates

Subtracting two dates gives a duration in days;
[`temporal_until()`](https://pedrobtz.github.io/zeitig/reference/temporal_until.md)
balances into larger units:

``` r

start <- plain_date(2024, 1, 15)
end <- plain_date(2024, 12, 25)
end - start
#> <duration[1]>
#> [1] P345D
temporal_until(start, end, largest_unit = "month")
#> <duration[1]>
#> [1] P11M10D
duration_total(end - start, "day")
#> [1] 345
duration_total(end - start, "week")
#> [1] 49.28571
```

## How old is someone?

``` r

birthdays <- plain_date(c("1990-02-28", "2000-02-29", "2010-12-31"))
today <- plain_date(2024, 2, 28)
age <- temporal_until(birthdays, today, largest_unit = "year")
age
#> <duration[3]>
#> [1] P34Y       P23Y11M30D P13Y1M28D
temporal_fields(age)$years
#> [1] 34 23 13
```

## First and last day of the month

``` r

d <- plain_date(c("2024-02-10", "2023-02-10", "2024-11-30"))
temporal_with(d, day = 1)
#> <plain_date[3]>
#> [1] 2024-02-01 2023-02-01 2024-11-01
temporal_with(d, day = days_in_month(d))
#> <plain_date[3]>
#> [1] 2024-02-29 2023-02-28 2024-11-30
```

## The next Monday, or the next given weekday

The next occurrence strictly after each date (a Monday maps to the
following Monday):

``` r

next_weekday <- function(date, weekday) {
  ahead <- (weekday - day_of_week(date) + 7) %% 7
  ahead[ahead == 0] <- 7
  date + duration(days = ahead)
}
d <- plain_date(2024, 7, 1:7)
data.frame(date = d, weekday = day_of_week(d), next_monday = next_weekday(d, 1))
#>         date weekday next_monday
#> 1 2024-07-01       1  2024-07-08
#> 2 2024-07-02       2  2024-07-08
#> 3 2024-07-03       3  2024-07-08
#> 4 2024-07-04       4  2024-07-08
#> 5 2024-07-05       5  2024-07-08
#> 6 2024-07-06       6  2024-07-08
#> 7 2024-07-07       7  2024-07-08
```

## Business hours and times of day

Plain times compare and sort as wall-clock times, and arithmetic on them
wraps at midnight:

``` r

opening <- plain_time(9)
closing <- plain_time(17, 30)
t <- plain_time(c("08:59", "12:00", "17:30", "21:15"))
t >= opening & t < closing
#> [1] FALSE  TRUE FALSE FALSE
plain_time(23, 30) + duration(hours = 2)
#> <plain_time[1]>
#> [1] 01:30:00
temporal_until(opening, closing)
#> <duration[1]>
#> [1] PT8H30M
```

## Rounding

``` r

t <- plain_time("14:37:52.123")
temporal_round(t, "minute")
#> <plain_time[1]>
#> [1] 14:38:00
temporal_round(t, "minute", rounding_increment = 15)
#> <plain_time[1]>
#> [1] 14:45:00
temporal_round(t, "hour", rounding_mode = "floor")
#> <plain_time[1]>
#> [1] 14:00:00

dt <- plain_date_time("2024-07-04T16:45:30")
temporal_round(dt, "day")
#> <plain_date_time[1]>
#> [1] 2024-07-05T00:00:00
```

Durations round too, balancing into larger units:

``` r

temporal_round(duration(minutes = 135), largest_unit = "hour")
#> <duration[1]>
#> [1] PT2H15M
temporal_round(duration(seconds = 5000), "minute", largest_unit = "hour")
#> <duration[1]>
#> [1] PT1H23M
```

## Comparing and sorting

Comparison operators, [`sort()`](https://rdrr.io/r/base/sort.html),
[`order()`](https://rdrr.io/r/base/order.html),
[`min()`](https://rdrr.io/r/base/Extremes.html)/[`max()`](https://rdrr.io/r/base/Extremes.html)
and [`unique()`](https://rdrr.io/r/base/unique.html) follow Temporal’s
ordering:

``` r

x <- plain_date(c("2024-05-01", "2023-12-31", "2024-01-15"))
sort(x)
#> <plain_date[3]>
#> [1] 2023-12-31 2024-01-15 2024-05-01
x > plain_date(2024, 1, 1)
#> [1]  TRUE FALSE  TRUE
temporal_compare(x, "2024-01-15")
#> [1]  1 -1  0
range(x)
#> <plain_date[2]>
#> [1] 2023-12-31 2024-05-01
```

Zoned date-times compare by the exact time, so the same moment in two
zones is equal:

``` r

berlin <- zoned_date_time("2024-06-01T12:00+02:00[Europe/Berlin]")
tokyo <- with_time_zone(berlin, "Asia/Tokyo")
berlin == tokyo
#> [1] TRUE
temporal_equals(berlin, tokyo)   # also compares the time zone
#> [1] FALSE
```

## Converting between time zones

``` r

meeting <- zoned_date_time(2024, 3, 15, 9, time_zone = "America/New_York")
with_time_zone(meeting, c("Europe/London", "Asia/Kolkata", "Australia/Sydney"))
#> <zoned_date_time[3]>
#> [1] 2024-03-15T13:00:00+00:00[Europe/London]   
#> [2] 2024-03-15T18:30:00+05:30[Asia/Kolkata]    
#> [3] 2024-03-16T00:00:00+11:00[Australia/Sydney]
```

A wall-clock time in one place, converted to the exact instant and back
to a local reading elsewhere:

``` r

local <- plain_date_time("2024-11-03T01:30")
to_zoned_date_time(local, "America/Chicago", disambiguation = "later")
#> <zoned_date_time[1]>
#> [1] 2024-11-03T01:30:00-06:00[America/Chicago]
to_instant(to_zoned_date_time(local, "America/Chicago"))
#> <instant[1]>
#> [1] 2024-11-03T06:30:00Z
```

## Daily schedules across daylight saving time

Adding days to a zoned date-time keeps the local time, even when the day
is 23 or 25 hours long. Adding hours is exact elapsed time:

``` r

first <- zoned_date_time("2024-03-08T09:00-05:00[America/New_York]")
first + duration(days = 0:3)
#> <zoned_date_time[4]>
#> [1] 2024-03-08T09:00:00-05:00[America/New_York]
#> [2] 2024-03-09T09:00:00-05:00[America/New_York]
#> [3] 2024-03-10T09:00:00-04:00[America/New_York]
#> [4] 2024-03-11T09:00:00-04:00[America/New_York]
first + duration(hours = 24 * (0:3))
#> <zoned_date_time[4]>
#> [1] 2024-03-08T09:00:00-05:00[America/New_York]
#> [2] 2024-03-09T09:00:00-05:00[America/New_York]
#> [3] 2024-03-10T10:00:00-04:00[America/New_York]
#> [4] 2024-03-11T10:00:00-04:00[America/New_York]
hours_in_day(first + duration(days = 0:3))
#> [1] 24 24 23 24
```

## Timestamps and epoch values

``` r

x <- instant_from_epoch(milliseconds = 1720000000000)
x
#> <instant[1]>
#> [1] 2024-07-03T09:46:40Z
format(epoch_seconds(x), scientific = FALSE)
#> [1] "1720000000"
format(epoch_milliseconds(x), scientific = FALSE)
#> [1] "1720000000000"
epoch_nanoseconds(instant("2024-01-01T00:00:00.123456789Z"))
#> [1] "1704067200123456789"
```

Differences between instants are exact and default to seconds:

``` r

a <- instant("2024-07-01T10:00:00Z")
b <- instant("2024-07-02T12:30:15.5Z")
b - a
#> <duration[1]>
#> [1] PT95415.5S
temporal_until(a, b, largest_unit = "hour")
#> <duration[1]>
#> [1] PT26H30M15.5S
duration_total(b - a, "hour")
#> [1] 26.50431
```

## Durations

``` r

d <- duration(c("PT90M", "P1DT2H", "PT45S"))
d
#> <duration[3]>
#> [1] PT90M  P1DT2H PT45S
duration_total(d, "minute")
#> [1]   90.00 1560.00    0.75
sort(d)
#> <duration[3]>
#> [1] PT45S  PT90M  P1DT2H
sum_duration <- Reduce(`+`, as.list(d))
sum_duration
#> <duration[1]>
#> [1] P1DT3H30M45S
abs(-duration(hours = 3))
#> <duration[1]>
#> [1] PT3H
```

Durations with months or years have no fixed length; give a starting
date to measure them:

``` r

duration_total(duration(months = 1), "day")
#> Error in `duration_total()`:
#> ! using unit 'month' in span or configuration requires that a relative reference time be given (`jiff::SpanRelativeTo::days_are_24_hours()` was given but this only permits using days and weeks without a relative reference time) (element 1)
duration_total(duration(months = 1), "day", relative_to = c("2024-01-01", "2024-02-01"))
#> [1] 31 29
```

## Formatting for display

``` r

z <- zoned_date_time("2024-07-04T16:45:30.123456789-04:00[America/New_York]")
format(z)
#> [1] "2024-07-04T16:45:30.123456789-04:00[America/New_York]"
format(z, smallest_unit = "minute", time_zone_name = "never")
#> [1] "2024-07-04T16:45-04:00"
format(z, fractional_second_digits = 3)
#> [1] "2024-07-04T16:45:30.123-04:00[America/New_York]"
temporal_strftime(z, "%A %e %B %Y, %H:%M %Z")
#> [1] "Thursday  4 July 2024, 16:45 EDT"
temporal_strftime(plain_date(2024, 7, 4), "%d.%m.%Y")
#> [1] "04.07.2024"
```

## Working with base R and data frames

Temporal vectors are ordinary vectors, so they work as data frame
columns:

``` r

events <- data.frame(
  id = 1:4,
  when = zoned_date_time(
    c(
      "2024-03-09T23:30-05:00[America/New_York]",
      "2024-03-10T08:00-04:00[America/New_York]",
      "2024-03-10T09:15+01:00[Europe/Paris]",
      "2024-03-11T07:45+09:00[Asia/Tokyo]"
    )
  )
)
events$date_utc <- to_plain_date(with_time_zone(events$when, "UTC"))
events$local_hour <- hour(events$when)
events[order(events$when), ]
#>   id                                        when   date_utc local_hour
#> 1  1 2024-03-09T23:30:00-05:00[America/New_York] 2024-03-10         23
#> 3  3     2024-03-10T09:15:00+01:00[Europe/Paris] 2024-03-10          9
#> 2  2 2024-03-10T08:00:00-04:00[America/New_York] 2024-03-10          8
#> 4  4       2024-03-11T07:45:00+09:00[Asia/Tokyo] 2024-03-10          7
summary(events$when)
#>                                        Min. 
#> 2024-03-09T23:30:00-05:00[America/New_York] 
#>                                     1st Qu. 
#> 2024-03-09T23:30:00-05:00[America/New_York] 
#>                                      Median 
#>     2024-03-10T09:15:00+01:00[Europe/Paris] 
#>                                     3rd Qu. 
#> 2024-03-10T08:00:00-04:00[America/New_York] 
#>                                        Max. 
#>       2024-03-11T07:45:00+09:00[Asia/Tokyo]
```

Conversions to and from base classes:

``` r

as_plain_date(as.Date("2024-02-29"))
#> <plain_date[1]>
#> [1] 2024-02-29
as.Date(plain_date(2024, 2, 29))
#> [1] "2024-02-29"
as_zoned_date_time(as.POSIXct("2024-07-04 12:00:00", tz = "Europe/Paris"))
#> <zoned_date_time[1]>
#> [1] 2024-07-04T12:00:00+02:00[Europe/Paris]
as.POSIXct(instant("2024-07-04T10:00:00Z"), tz = "UTC")
#> [1] "2024-07-04 10:00:00 UTC"
as_duration(as.difftime(90, units = "mins"))
#> <duration[1]>
#> [1] PT90M
as_difftime(duration(hours = 36), units = "days")
#> Time difference of 1.5 days
```

`POSIXct` stores seconds in a double, so conversions through it are
rounded to the microsecond; keep values in zeitig types when nanoseconds
matter.
