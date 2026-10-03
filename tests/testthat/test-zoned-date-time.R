test_that("zoned_date_time() from components and strings", {
  x <- zoned_date_time(1995, 12, 7, 3, 24, 30, time_zone = "America/New_York")
  expect_s3_class(x, "zudate_zoned_date_time")
  expect_equal(format(x), "1995-12-07T03:24:30-05:00[America/New_York]")
  expect_equal(
    format(zoned_date_time("1995-12-07T03:24:30-08:00[America/Los_Angeles]")),
    "1995-12-07T03:24:30-08:00[America/Los_Angeles]"
  )
  expect_equal(
    format(zoned_date_time(
      2020, 1, 1,
      time_zone = c("UTC", "asia/tokyo", "+05:30", "-0800", "Etc/UTC")
    )),
    c(
      "2020-01-01T00:00:00+00:00[UTC]", "2020-01-01T00:00:00+09:00[Asia/Tokyo]",
      "2020-01-01T00:00:00+05:30[+05:30]", "2020-01-01T00:00:00-08:00[-08:00]",
      "2020-01-01T00:00:00+00:00[Etc/UTC]"
    )
  )
  expect_equal(time_zone(zoned_date_time(2020, 1, 1, time_zone = "europe/paris")), "Europe/Paris")
  # critical flag annotation
  expect_equal(
    format(zoned_date_time("2020-01-01T00:00+01:00[!Europe/Paris]")),
    "2020-01-01T00:00:00+01:00[Europe/Paris]"
  )
})

test_that("invalid input", {
  expect_error(
    zoned_date_time(2020, 1, 1, time_zone = "Mars/Base"),
    "unknown time zone",
    class = "zudate_range_error"
  )
  expect_error(zoned_date_time(2020, 1, 1, time_zone = "+05:30:15"), class = "zudate_range_error")
  expect_error(
    zoned_date_time(2020, 1, 1, time_zone = "EST5EDT,M3.2.0,M11.1.0"),
    class = "zudate_range_error"
  )
  expect_error(zoned_date_time(2020, 1, 1), class = "zudate_type_error")
  expect_error(zoned_date_time(2020, 1, 1, time_zone = 1), class = "zudate_type_error")
  expect_error(zoned_date_time("2020-01-01T00:00+01:00"), class = "zudate_range_error")
  expect_error(zoned_date_time("2020-01-01T00:00Z"), class = "zudate_range_error")
})

test_that("DST gaps and overlaps in three zones", {
  cases <- list(
    # zone, gap (local), overlap (local)
    list("America/New_York", c(2019, 3, 10, 2, 30), c(2019, 11, 3, 1, 30), "-05:00", "-04:00"),
    list("Europe/London", c(2019, 3, 31, 1, 30), c(2019, 10, 27, 1, 30), "+00:00", "+01:00"),
    list("Australia/Sydney", c(2019, 10, 6, 2, 30), c(2019, 4, 7, 2, 30), "+10:00", "+11:00")
  )
  for (case in cases) {
    tz <- case[[1]]
    g <- case[[2]]
    o <- case[[3]]
    std <- case[[4]]
    dst <- case[[5]]
    gap <- function(d) {
      zoned_date_time(g[1], g[2], g[3], g[4], g[5], time_zone = tz, disambiguation = d)
    }
    over <- function(d) {
      zoned_date_time(o[1], o[2], o[3], o[4], o[5], time_zone = tz, disambiguation = d)
    }
    # gap: compatible/later move forward, earlier moves back
    expect_equal(offset(gap("compatible")), dst, info = tz)
    expect_equal(hour(gap("compatible")), g[4] + 1, info = tz)
    expect_equal(hour(gap("later")), g[4] + 1, info = tz)
    expect_equal(hour(gap("earlier")), g[4] - 1, info = tz)
    expect_error(gap("reject"), class = "zudate_range_error")
    # overlap: compatible/earlier take the first (DST) offset
    expect_equal(offset(over("compatible")), dst, info = tz)
    expect_equal(offset(over("earlier")), dst, info = tz)
    expect_equal(offset(over("later")), std, info = tz)
    expect_equal(hour(over("later")), o[4], info = tz)
    expect_error(over("reject"), class = "zudate_range_error")
    expect_true(hours_in_day(gap("compatible")) == 23)
    expect_true(hours_in_day(over("compatible")) == 25)
  }
})

test_that("the offset option when parsing", {
  s <- "2020-01-01T00:00+01:00[America/New_York]"
  expect_error(zoned_date_time(s), class = "zudate_range_error")
  expect_equal(
    format(zoned_date_time(s, offset = "ignore")),
    "2020-01-01T00:00:00-05:00[America/New_York]"
  )
  expect_equal(
    format(zoned_date_time(s, offset = "use")),
    "2019-12-31T18:00:00-05:00[America/New_York]"
  )
  expect_equal(
    format(zoned_date_time(s, offset = "prefer")),
    "2020-01-01T00:00:00-05:00[America/New_York]"
  )
  # offset disambiguates an overlap
  later <- zoned_date_time("2019-11-03T01:30-05:00[America/New_York]")
  expect_equal(offset(later), "-05:00")
})

test_that("fields and time zone information", {
  z <- zoned_date_time("2020-03-08T12:00:00.123-04:00[America/New_York]")
  expect_equal(time_zone(z), "America/New_York")
  expect_equal(offset(z), "-04:00")
  expect_equal(offset_nanoseconds(z), -4 * 3600 * 1e9)
  expect_equal(c(year(z), month(z), day(z), hour(z), millisecond(z)), c(2020L, 3L, 8L, 12L, 123L))
  expect_equal(day_of_week(z), 7L)
  expect_equal(hours_in_day(z), 23)
  expect_equal(format(start_of_day(z)), "2020-03-08T00:00:00-05:00[America/New_York]")
  expect_equal(
    format(time_zone_transition(z, "previous")),
    "2020-03-08T03:00:00-04:00[America/New_York]"
  )
  expect_equal(
    format(time_zone_transition(z, "next")),
    "2020-11-01T01:00:00-05:00[America/New_York]"
  )
  expect_true(is.na(time_zone_transition(zoned_date_time(2020, 1, 1, time_zone = "UTC"))))
  expect_equal(format(with_time_zone(z, "Asia/Tokyo")), "2020-03-09T01:00:00.123+09:00[Asia/Tokyo]")
  expect_equal(temporal_fields(z)$hour, 12L)
  # Beirut starts the DST day at 01:00
  b <- to_zoned_date_time(plain_date(2020, 3, 29), "Asia/Beirut")
  expect_equal(format(b), "2020-03-29T01:00:00+03:00[Asia/Beirut]")
  # historical sub-minute offsets are printed with seconds
  expect_equal(offset(zoned_date_time(1880, 1, 1, time_zone = "America/New_York")), "-04:56:02")
})

test_that("conversions", {
  z <- zoned_date_time("2020-03-08T12:00-04:00[America/New_York]")
  expect_equal(format(to_instant(z)), "2020-03-08T16:00:00Z")
  expect_equal(format(to_plain_date(z)), "2020-03-08")
  expect_equal(format(to_plain_time(z)), "12:00:00")
  expect_equal(format(to_plain_date_time(z)), "2020-03-08T12:00:00")
  expect_equal(
    format(to_zoned_date_time(instant("2020-01-01T00:00Z"), "Asia/Tokyo")),
    "2020-01-01T09:00:00+09:00[Asia/Tokyo]"
  )
  expect_equal(
    format(to_zoned_date_time(
      plain_date_time("2019-03-10T02:30"), "America/New_York",
      disambiguation = "earlier"
    )),
    "2019-03-10T01:30:00-05:00[America/New_York]"
  )
  expect_equal(
    format(to_zoned_date_time(plain_date(2020, 1, 1), "UTC", time = plain_time(12))),
    "2020-01-01T12:00:00+00:00[UTC]"
  )
  expect_equal(format(to_zoned_date_time(z, "UTC")), "2020-03-08T16:00:00+00:00[UTC]")
  expect_equal(as.Date(z), as.Date("2020-03-08"))
  expect_equal(format(as_plain_date_time(z)), "2020-03-08T12:00:00")
  expect_error(to_instant(plain_date(2020, 1, 1)), class = "zudate_type_error")
})

test_that("temporal_with() and with_plain_*()", {
  o <- zoned_date_time("2019-11-03T01:30-05:00[America/New_York]")
  # prefer keeps the second 01:45 in the overlap
  expect_equal(format(temporal_with(o, minute = 45)), "2019-11-03T01:45:00-05:00[America/New_York]")
  expect_equal(
    format(temporal_with(o, minute = 45, offset = "ignore")),
    "2019-11-03T01:45:00-04:00[America/New_York]"
  )
  expect_equal(format(temporal_with(o, day = 4)), "2019-11-04T01:30:00-05:00[America/New_York]")
  expect_equal(format(with_plain_time(o)), "2019-11-03T00:00:00-04:00[America/New_York]")
  expect_equal(format(with_plain_time(o, "12:00")), "2019-11-03T12:00:00-05:00[America/New_York]")
  expect_equal(
    format(with_plain_date(o, "2019-11-10")),
    "2019-11-10T01:30:00-05:00[America/New_York]"
  )
})

test_that("calendar-aware arithmetic", {
  z <- zoned_date_time("2020-03-07T12:00-05:00[America/New_York]")
  expect_equal(format(z + duration(days = 1)), "2020-03-08T12:00:00-04:00[America/New_York]")
  expect_equal(format(z + duration(hours = 24)), "2020-03-08T13:00:00-04:00[America/New_York]")
  next_day <- z + duration(days = 1)
  expect_equal(format(next_day - z), "PT23H")
  expect_equal(format(temporal_until(z, next_day, largest_unit = "day")), "P1D")
  expect_equal(format(temporal_until(z, next_day, largest_unit = "minute")), "PT1380M")
  expect_error(
    temporal_until(z, with_time_zone(next_day, "UTC"), largest_unit = "day"),
    class = "zudate_range_error"
  )
  expect_equal(format(temporal_until(z, with_time_zone(next_day, "UTC"))), "PT23H")
  jan31 <- zoned_date_time("2021-01-31T10:00+01:00[Europe/Paris]")
  expect_equal(format(jan31 + duration(months = 1)), "2021-02-28T10:00:00+01:00[Europe/Paris]")
  expect_error(temporal_add(jan31, "P1M", overflow = "reject"), class = "zudate_range_error")
  # 11 of the 23 hours of 2020-03-08 have elapsed at noon: rounds down
  expect_equal(
    format(temporal_round(next_day, "day")),
    "2020-03-08T00:00:00-05:00[America/New_York]"
  )
  expect_equal(
    format(temporal_round(zoned_date_time("2020-03-08T13:00-04:00[America/New_York]"), "day")),
    "2020-03-09T00:00:00-04:00[America/New_York]"
  )
  expect_equal(
    format(temporal_round(zoned_date_time("2020-03-08T11:00-04:00[America/New_York]"), "day")),
    "2020-03-08T00:00:00-05:00[America/New_York]"
  )
  expect_equal(
    format(temporal_round(z, "hour", rounding_increment = 6)),
    "2020-03-07T12:00:00-05:00[America/New_York]"
  )
  # durations relative to a zoned date-time follow DST
  expect_equal(duration_total(duration(days = 1), "hour", relative_to = z), 23)
  expect_equal(
    duration_total(
      duration(days = 1), "hour",
      relative_to = "2020-03-07T12:00-05:00[America/New_York]"
    ),
    23
  )
  expect_equal(
    format(temporal_round(duration(hours = 23), largest_unit = "day", relative_to = z)),
    "P1D"
  )
})

test_that("comparison uses the exact time; temporal_equals() also the zone", {
  a <- zoned_date_time("2020-01-01T00:00+01:00[Europe/Paris]")
  b <- with_time_zone(a, "UTC")
  expect_true(a == b)
  expect_false(temporal_equals(a, b))
  expect_true(temporal_equals(a, a))
  expect_equal(temporal_compare(a, b), 0L)
  x <- c(a, zoned_date_time("2019-12-31T23:30Z[UTC]"), b)
  expect_equal(order(x), c(1L, 3L, 2L))
  expect_length(unique(x), 2)
  expect_equal(temporal_compare(a, "2020-01-01T00:00+00:00[UTC]"), -1L)
})

test_that("NA, zero-length and vctrs", {
  x <- zoned_date_time(c(2020, NA), 1, 1, time_zone = c("UTC", "Asia/Tokyo"))
  expect_equal(is.na(x), c(FALSE, TRUE))
  expect_equal(format(x), c("2020-01-01T00:00:00+00:00[UTC]", NA))
  expect_equal(time_zone(x), c("UTC", NA))
  expect_equal(offset(x), c("+00:00", NA))
  expect_equal(hours_in_day(x), c(24, NA))
  expect_equal(hour(x), c(0L, NA))
  expect_true(is.na(zoned_date_time(2020, 1, 1, time_zone = NA_character_)))
  expect_length(zoned_date_time(character()), 0)
  expect_equal(format(x + duration(days = 1))[2], NA_character_)
  path <- tempfile(fileext = ".rds")
  saveRDS(x, path)
  expect_equal(readRDS(path), x)
  expect_equal(data.frame(x = x)$x, x)
  expect_snapshot(zoned_date_time(c("2020-01-01T00:00+01:00[Europe/Paris]", NA)))
})

test_that("POSIXct interop", {
  ct <- as.POSIXct("2020-07-01 12:00:00", tz = "Europe/Paris")
  z <- as_zoned_date_time(ct)
  expect_equal(format(z), "2020-07-01T12:00:00+02:00[Europe/Paris]")
  expect_equal(as.POSIXct(z), ct)
  expect_equal(format(as_zoned_date_time(ct, time_zone = "UTC")), "2020-07-01T10:00:00+00:00[UTC]")
  mixed <- zoned_date_time(2020, 1, 1, time_zone = c("UTC", "Asia/Tokyo"))
  expect_equal(attr(as.POSIXct(mixed), "tzone"), "UTC")
  expect_equal(attr(as.POSIXct(zoned_date_time(2020, 1, 1, time_zone = "+05:30")), "tzone"), "UTC")
  expect_equal(as.POSIXlt(z)$hour, 12L)
  expect_equal(format(as_instant(z)), "2020-07-01T10:00:00Z")
})
