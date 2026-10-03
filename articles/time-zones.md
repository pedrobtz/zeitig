# Time zones

``` r

library(zeitig)
#> 
#> Attaching package: 'zeitig'
#> The following object is masked from 'package:stats':
#> 
#>     offset
```

## Exact time versus wall-clock time

An [`instant()`](https://pedrobtz.github.io/zeitig/reference/instant.md)
is a point on the global timeline. A
[`zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/zoned_date_time.md)
is the same point plus a time zone, which gives it a wall-clock reading.
Plain types have a wall-clock reading but no position on the timeline.

``` r

i <- instant("2020-03-08T16:00Z")
with_time_zone(to_zoned_date_time(i, "UTC"), c("America/New_York", "Asia/Tokyo", "+05:30"))
#> <zoned_date_time[3]>
#> [1] 2020-03-08T12:00:00-04:00[America/New_York]
#> [2] 2020-03-09T01:00:00+09:00[Asia/Tokyo]      
#> [3] 2020-03-08T21:30:00+05:30[+05:30]
```

Each element carries its own time zone. Identifiers are IANA names (see
[`available_time_zones()`](https://pedrobtz.github.io/zeitig/reference/available_time_zones.md))
or fixed offsets such as `"+05:30"`.

## Where the database comes from

On Linux and macOS zeitig reads the operating system’s time zone
database (`/usr/share/zoneinfo`, or the directory in `TZDIR`), so its
rules are as current as the system’s. On Windows, which has no such
database, the copy of the IANA database bundled with the jiff crate is
used. Setting `ZEITIG_TZDIR` before the first time zone lookup in a
session points zeitig at another zoneinfo directory.

The default time zone, used by
[`now_zoned_date_time()`](https://pedrobtz.github.io/zeitig/reference/now_instant.md)
and for `POSIXct` values without a `tzone`, is the `TZ` environment
variable when set, otherwise
[`Sys.timezone()`](https://rdrr.io/r/base/timezones.html):

``` r

now_time_zone()
#> [1] "UTC"
```

## Daylight saving time

When clocks spring forward, some local times do not exist; when they
fall back, some occur twice. `disambiguation` decides which exact time
such a wall-clock time means:

``` r

# 02:30 did not exist in New York on 2019-03-10
gap <- plain_date_time("2019-03-10T02:30")
for (d in c("compatible", "earlier", "later")) {
  print(to_zoned_date_time(gap, "America/New_York", disambiguation = d))
}
#> <zoned_date_time[1]>
#> [1] 2019-03-10T03:30:00-04:00[America/New_York]
#> <zoned_date_time[1]>
#> [1] 2019-03-10T01:30:00-05:00[America/New_York]
#> <zoned_date_time[1]>
#> [1] 2019-03-10T03:30:00-04:00[America/New_York]
```

``` r

to_zoned_date_time(gap, "America/New_York", disambiguation = "reject")
#> Error in `to_zoned_date_time()`:
#> ! error converting datetime to instant in time zone America/New_York: datetime is ambiguous since it falls into a gap between offsets -05 and -04 (element 1)
```

Arithmetic with calendar units follows the wall clock, while time units
are exact:

``` r

z <- zoned_date_time("2020-03-07T12:00-05:00[America/New_York]")
z + duration(days = 1)
#> <zoned_date_time[1]>
#> [1] 2020-03-08T12:00:00-04:00[America/New_York]
z + duration(hours = 24)
#> <zoned_date_time[1]>
#> [1] 2020-03-08T13:00:00-04:00[America/New_York]
hours_in_day(z + duration(days = 1))
#> [1] 23
temporal_until(z, z + duration(days = 1))
#> <duration[1]>
#> [1] PT23H
```

Changing a field of a zoned date-time inside a DST overlap keeps its UTC
offset when possible (`offset = "prefer"`):

``` r

o <- zoned_date_time("2019-11-03T01:30-05:00[America/New_York]")
temporal_with(o, minute = 45)
#> <zoned_date_time[1]>
#> [1] 2019-11-03T01:45:00-05:00[America/New_York]
temporal_with(o, minute = 45, offset = "ignore")
#> <zoned_date_time[1]>
#> [1] 2019-11-03T01:45:00-04:00[America/New_York]
```

Transitions can be found directly:

``` r

time_zone_transition(z, "next")
#> <zoned_date_time[1]>
#> [1] 2020-03-08T03:00:00-04:00[America/New_York]
start_of_day(zoned_date_time("2020-03-29T12:00+03:00[Asia/Beirut]"))
#> <zoned_date_time[1]>
#> [1] 2020-03-29T01:00:00+03:00[Asia/Beirut]
```

## Parsing strings with offsets

A string like `2020-01-01T00:00+01:00[America/New_York]` is
inconsistent: New York is not at `+01:00` on that date. By default that
is an error; `offset` chooses another resolution.

``` r

s <- "2020-01-01T00:00+01:00[America/New_York]"
zoned_date_time(s)
#> Error in `zoned_date_time()`:
#> ! datetime could not resolve to a timestamp since `reject` conflict resolution was chosen, and because datetime has offset `+01`, but the time zone `America/New_York` for the given datetime unambiguously has offset `-05` (element 1)
zoned_date_time(s, offset = "use")
#> <zoned_date_time[1]>
#> [1] 2019-12-31T18:00:00-05:00[America/New_York]
zoned_date_time(s, offset = "ignore")
#> <zoned_date_time[1]>
#> [1] 2020-01-01T00:00:00-05:00[America/New_York]
```
