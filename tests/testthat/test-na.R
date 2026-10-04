# `NA` at the R/Rust boundary: every entry point must turn a missing element
# into a missing result (never an error or a crash), in every argument that
# carries data, and keep the other elements intact.

expect_na_at <- function(x, at = 2L) {
  miss <- if (is.data.frame(x)) is.na(x[[1]]) else is.na(x)
  expect_identical(which(miss), as.integer(at))
}

test_that("plain dates propagate NA through every operation", {
  x <- plain_date(c("2020-01-31", NA, "2021-06-15"))
  d <- duration(c("P1M", "P1D", NA))
  expect_na_at(x)
  expect_na_at(plain_date(c(2020, NA, 2021), 1, 1))
  expect_na_at(plain_date(2020, c(1, 1, NA), 1), 3)
  expect_na_at(format(x))
  expect_na_at(format(x, calendar_name = "always"))
  expect_na_at(as.Date(x))
  expect_na_at(as_plain_date(as.Date(c("2020-01-01", NA))))
  for (f in list(year, month, day, day_of_week, day_of_year, week_of_year, year_of_week,
                 days_in_week, days_in_month, days_in_year, months_in_year, in_leap_year)) {
    expect_na_at(f(x))
  }
  expect_na_at(temporal_fields(x))
  expect_na_at(x + d, 2:3)
  expect_na_at(temporal_add(x, duration(days = c(1, 1, NA)), overflow = "reject"), 2:3)
  expect_na_at(temporal_until(x, rev(x)), 2)
  expect_na_at(temporal_since(x, x, largest_unit = "year", rounding_mode = "halfEven"))
  expect_na_at(temporal_with(x, day = 1))
  expect_na_at(temporal_with(x[c(1, 3)], day = c(NA, 1)), 1)
  expect_na_at(temporal_compare(x, x))
  expect_na_at(temporal_equals(x, x))
  expect_na_at(to_plain_date_time(x))
  expect_na_at(to_zoned_date_time(x, "Europe/Paris"))
  expect_na_at(to_zoned_date_time(x, c("UTC", "UTC", NA)), 2:3)
  expect_na_at(temporal_strftime(x, "%Y"))
  expect_na_at(temporal_strftime(x, c("%Y", "%Y", NA)), 2:3)
  expect_na_at(temporal_strptime(c("2020-01-01", NA, "2020-01-02"), "%Y-%m-%d", "plain_date"))
})

test_that("plain times and date-times propagate NA through every operation", {
  t <- plain_time(c("12:00", NA, "23:59:59.5"))
  dt <- plain_date_time(c("2020-01-01T12:00", NA, "2020-03-01T00:00:00.5"))
  d <- duration(hours = c(1, 1, NA))
  for (x in list(t, dt)) {
    expect_na_at(x)
    expect_na_at(format(x))
    expect_na_at(format(x, fractional_second_digits = 3, rounding_mode = "halfEven"))
    expect_na_at(format(x, smallest_unit = "minute"))
    for (f in list(hour, minute, second, millisecond, microsecond, nanosecond)) {
      expect_na_at(f(x))
    }
    expect_na_at(temporal_fields(x))
    expect_na_at(x + d, 2:3)
    expect_na_at(temporal_until(x, x, smallest_unit = "second", rounding_mode = "halfEven"))
    expect_na_at(temporal_round(x, "second", rounding_mode = "halfEven"))
    expect_na_at(temporal_with(x, hour = 1))
    expect_na_at(temporal_strftime(x, "%H"))
  }
  expect_na_at(plain_time(c(1, NA, 3)))
  expect_na_at(plain_date_time(2020, 1, 1, c(1, NA, 3)))
  expect_na_at(plain_date_time(c(2020, NA, 2021), 1, 1, 12))
  expect_na_at(as.POSIXct(dt))
  expect_na_at(as.POSIXlt(dt))
  expect_na_at(as_plain_date_time(as.POSIXct(c("2020-01-01", NA, "2020-01-02"), tz = "UTC")))
  expect_na_at(to_plain_date(dt))
  expect_na_at(to_plain_time(dt))
  expect_na_at(with_plain_time(dt, plain_time(9)))
  expect_na_at(with_plain_date(dt, plain_date(2000, 1, 1)))
  expect_na_at(to_zoned_date_time(dt, "America/New_York", disambiguation = "reject"))
})

test_that("instants and zoned date-times propagate NA through every operation", {
  i <- instant(c("2020-01-01T00:00Z", NA, "1969-12-31T23:59:59.5Z"))
  z <- zoned_date_time(c(
    "2020-03-08T12:00-04:00[America/New_York]", NA, "2020-01-01T00:00+01:00[Europe/Paris]"
  ))
  d <- duration(hours = c(1, 1, NA))
  for (x in list(i, z)) {
    expect_na_at(x)
    expect_na_at(format(x))
    expect_na_at(format(x, fractional_second_digits = 2, rounding_mode = "halfExpand"))
    expect_na_at(epoch_seconds(x))
    expect_na_at(epoch_milliseconds(x))
    expect_na_at(epoch_nanoseconds(x))
    expect_na_at(x + d, 2:3)
    expect_na_at(temporal_until(x, x, largest_unit = "hour", rounding_mode = "halfEven"))
    expect_na_at(temporal_round(x, "minute", rounding_mode = "halfEven"))
    expect_na_at(temporal_strftime(x, "%Y"))
    expect_na_at(as.POSIXct(x))
  }
  # an NA time zone prints the instant in UTC
  expect_na_at(format(i, time_zone = c("Asia/Tokyo", "UTC", NA)), 2)
  expect_na_at(format(i, time_zone = "Asia/Tokyo"))
  expect_na_at(instant_from_epoch(seconds = c(0, NA, 1)))
  expect_na_at(instant_from_epoch(milliseconds = c(0, NA, 1)))
  expect_na_at(instant_from_epoch(nanoseconds = c("0", NA, "1")))
  expect_na_at(instant_from_epoch(nanoseconds = c(0, NA, 1)))
  expect_na_at(to_zoned_date_time(i, "UTC"))
  expect_na_at(to_zoned_date_time(i[c(1, 3)], c("UTC", NA)), 2)
  expect_na_at(temporal_strptime(
    c("2020-01-01 00:00 +0100", NA, "2020-01-01 00:00 +0000"), "%Y-%m-%d %H:%M %z", "instant"
  ))
  expect_na_at(temporal_strptime(
    c("2020-01-01 00:00 Europe/Paris", NA, "2020-01-01 00:00 UTC"), "%Y-%m-%d %H:%M %Q",
    "zoned_date_time"
  ))
  for (f in list(year, hour, nanosecond, offset, offset_nanoseconds, time_zone, hours_in_day,
                 temporal_fields, to_instant, to_plain_date_time, start_of_day)) {
    expect_na_at(f(z))
  }
  expect_na_at(time_zone_transition(z, "next"))
  expect_na_at(time_zone_transition(z, "previous"))
  expect_na_at(with_time_zone(z, "Asia/Tokyo"))
  expect_na_at(with_time_zone(z[c(1, 3)], c(NA, "UTC")), 1)
  expect_na_at(temporal_with(z, hour = 1, offset = "use"))
  expect_na_at(with_plain_time(z, plain_time(9)))
  expect_na_at(zoned_date_time(2020, 1, 1, time_zone = c("UTC", NA, "UTC")))
  expect_na_at(temporal_equals(z, z))
})

test_that("durations propagate NA through every operation", {
  d <- duration(c("P1Y2M", NA, "PT36H"))
  rel <- plain_date(c("2020-01-31", "2020-01-01", NA))
  expect_na_at(d)
  expect_na_at(duration(hours = c(1, NA, 2)))
  expect_na_at(format(d))
  expect_na_at(format(d, fractional_second_digits = 3))
  expect_na_at(-d)
  expect_na_at(abs(d))
  expect_na_at(sign(d))
  expect_na_at(duration_blank(d))
  expect_na_at(temporal_fields(d))
  expect_na_at(temporal_with(d, years = 0))
  expect_na_at(d[2:3] + duration(hours = 1), 1)
  expect_na_at(duration_total(d, "hour", relative_to = rel), 2:3)
  expect_na_at(duration_compare(d, d, relative_to = rel), 2:3)
  expect_na_at(temporal_round(d, "day", relative_to = rel, rounding_mode = "halfEven"), 2:3)
  zrel <- zoned_date_time(c("2020-01-01T00:00Z[UTC]", NA, "2020-01-01T00:00Z[UTC]"))
  expect_na_at(duration_total(d, "day", relative_to = zrel))
  expect_na_at(vec_sort(d[2:3]), 2)
  expect_na_at(as_difftime(d[2:3]), 1)
})
