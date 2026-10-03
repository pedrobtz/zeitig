# Time zone information

- `time_zone()`: the time zone identifier of each element.

- `offset()`, `offset_nanoseconds()`: the UTC offset in effect, as a
  `"+HH:MM"` string or a number of nanoseconds.

- `hours_in_day()`: the length of the calendar day in hours (23 or 25 on
  DST transition days in many zones).

- `start_of_day()`: the first instant of the day (not always midnight).

- `time_zone_transition()`: the next or previous UTC offset change, `NA`
  when there is none.

- `with_time_zone()`: the same exact time viewed in another time zone.

## Usage

``` r
time_zone(x)

offset(x)

offset_nanoseconds(x)

hours_in_day(x)

start_of_day(x)

time_zone_transition(x, direction = c("next", "previous"))

with_time_zone(x, time_zone)
```

## Arguments

- x:

  A zoned date-time.

- direction:

  `"next"` or `"previous"`.

- time_zone:

  Time zone identifiers, recycled against `x`.

## Value

See above.

## See also

Other zoned date-time:
[`zoned_date_time()`](https://pedrobtz.github.io/zudate/reference/zoned_date_time.md)

## Examples

``` r
x <- zoned_date_time("2020-03-08T12:00-04:00[America/New_York]")
time_zone(x)
#> [1] "America/New_York"
offset(x)
#> [1] "-04:00"
hours_in_day(x)
#> [1] 23
start_of_day(x)
#> <zoned_date_time[1]>
#> [1] 2020-03-08T00:00:00-05:00[America/New_York]
time_zone_transition(x, "previous")
#> <zoned_date_time[1]>
#> [1] 2020-03-08T03:00:00-04:00[America/New_York]
time_zone_transition(x, "next")
#> <zoned_date_time[1]>
#> [1] 2020-11-01T01:00:00-05:00[America/New_York]
with_time_zone(x, "Asia/Tokyo")
#> <zoned_date_time[1]>
#> [1] 2020-03-09T01:00:00+09:00[Asia/Tokyo]
```
