test_that("adding to plain dates follows Temporal", {
  d <- plain_date(2021, 1, 31)
  expect_equal(format(temporal_add(d, duration(months = 1))), "2021-02-28")
  expect_equal(format(temporal_add(d, "P1M")), "2021-02-28")
  expect_error(
    temporal_add(d, "P1M", overflow = "reject"),
    "out of range",
    class = "zudate_range_error"
  )
  expect_equal(
    format(temporal_add(plain_date(2021, 1, 15), "P1M", overflow = "reject")),
    "2021-02-15"
  )
  expect_equal(format(d + duration(days = 1:3)), c("2021-02-01", "2021-02-02", "2021-02-03"))
  expect_equal(format(duration(days = 1) + d), "2021-02-01")
  expect_equal(format(d - duration(years = 1, days = 31)), "2019-12-31")
  expect_equal(format(temporal_subtract(d, "P1M")), "2020-12-31")
  # time units are balanced into whole days, the rest is ignored
  expect_equal(format(plain_date(2021, 1, 1) + duration(hours = 47)), "2021-01-02")
  expect_equal(format(plain_date(2020, 2, 29) + duration(years = 1)), "2021-02-28")
  expect_error(plain_date(9999, 12, 31) + duration(days = 1), class = "zudate_range_error")
})

test_that("adding to plain times wraps", {
  expect_equal(format(plain_time(23, 30) + duration(hours = 1)), "00:30:00")
  expect_equal(format(plain_time(0, 30) - duration(hours = 1)), "23:30:00")
  expect_equal(format(plain_time(12) + duration(days = 3, minutes = 1)), "12:01:00")
  expect_equal(format(plain_time(12) + duration(nanoseconds = 1)), "12:00:00.000000001")
})

test_that("adding to plain date-times", {
  dt <- plain_date_time("2020-02-29T12:00")
  expect_equal(format(dt - duration(years = 1)), "2019-02-28T12:00:00")
  expect_equal(format(dt + duration(hours = 13)), "2020-03-01T01:00:00")
  expect_error(temporal_add(dt, "P1Y", overflow = "reject"), class = "zudate_range_error")
  expect_equal(format(temporal_add(dt, "P1Y", overflow = "constrain")), "2021-02-28T12:00:00")
})

test_that("NA and zero-length arithmetic", {
  expect_true(is.na(plain_date(NA, 1, 1) + duration(days = 1)))
  expect_true(is.na(plain_date(2020, 1, 1) + duration(days = NA)))
  expect_length(plain_date(character()) + duration(days = 1), 0)
  expect_equal(format(temporal_until(plain_date(c("2020-01-01", NA)), "2020-01-31")), c("P30D", NA))
  expect_length(temporal_until(plain_time(character()), plain_time(character())), 0)
})

test_that("until / since on plain dates", {
  a <- plain_date(2006, 8, 24)
  b <- plain_date(2019, 1, 31)
  expect_equal(format(temporal_until(a, b)), "P4543D")
  expect_equal(format(temporal_until(a, b, largest_unit = "year")), "P12Y5M7D")
  expect_equal(format(temporal_until(a, b, largest_unit = "months")), "P149M7D")
  expect_equal(format(temporal_until(a, b, largest_unit = "week")), "P649W")
  expect_equal(format(temporal_since(a, b)), "-P4543D")
  expect_equal(format(b - a), "P4543D")
  expect_equal(
    format(temporal_until(a, b, smallest_unit = "month", largest_unit = "year")),
    "P12Y5M"
  )
  expect_equal(
    format(temporal_until(a, b, smallest_unit = "year", rounding_mode = "halfExpand")),
    "P12Y"
  )
  expect_error(temporal_until(a, b, smallest_unit = "hour"), class = "zudate_range_error")
  expect_error(temporal_until(a, plain_time(1)))
})

test_that("until / since on plain times and date-times", {
  expect_equal(format(temporal_since(plain_time(19, 39), plain_time(9, 0))), "PT10H39M")
  expect_equal(format(plain_time(9) - plain_time(19, 39)), "-PT10H39M")
  expect_equal(
    format(temporal_until(plain_time(9), plain_time(19, 39), largest_unit = "minute")),
    "PT639M"
  )
  a <- plain_date_time("2020-01-01T00:00")
  b <- plain_date_time("2020-01-02T13:31")
  expect_equal(format(temporal_until(a, b)), "P1DT13H31M")
  expect_equal(format(temporal_until(a, b, largest_unit = "hour")), "PT37H31M")
  expect_equal(
    format(temporal_until(a, b, smallest_unit = "hour", rounding_mode = "halfExpand")),
    "P1DT14H"
  )
  expect_equal(
    format(temporal_until(a, b, smallest_unit = "minute", rounding_increment = 15)),
    "P1DT13H30M"
  )
  expect_equal(format(b - a), "P1DT13H31M")
})

test_that("rounding plain times and date-times", {
  t <- plain_time(19, 39, 9, 68, 346, 205)
  expect_equal(format(temporal_round(t, "hour")), "20:00:00")
  expect_equal(
    format(temporal_round(t, "minute", rounding_increment = 15, rounding_mode = "floor")),
    "19:30:00"
  )
  expect_equal(format(temporal_round(t, "millisecond")), "19:39:09.068")
  expect_equal(format(temporal_round(plain_time(23, 59, 59, 500), "second")), "00:00:00")
  expect_error(temporal_round(t, "minute", rounding_increment = 7), class = "zudate_range_error")
  dt <- plain_date_time("1995-12-07T03:24:30.000003500")
  expect_equal(format(temporal_round(dt, "second")), "1995-12-07T03:24:30")
  expect_equal(format(temporal_round(dt, "day")), "1995-12-07T00:00:00")
  expect_equal(
    format(temporal_round(plain_date_time("1995-12-07T12:00"), "day")),
    "1995-12-08T00:00:00"
  )
  expect_error(temporal_round(t), class = "zudate_type_error")
  expect_error(temporal_round(plain_date(2020, 1, 1), "day"), class = "zudate_type_error")
  expect_error(temporal_round(t, "hour", relative_to = "2020-01-01"), class = "zudate_type_error")
})

test_that("round trip a + (b - a) == b", {
  set.seed(1)
  a <- plain_date(sample(1900:2100, 200, TRUE), sample(1:12, 200, TRUE), sample(1:28, 200, TRUE))
  b <- plain_date(sample(1900:2100, 200, TRUE), sample(1:12, 200, TRUE), sample(1:31, 200, TRUE))
  for (unit in c("year", "month", "week", "day")) {
    expect_equal(a + temporal_until(a, b, largest_unit = unit), b)
  }
})

test_that("unsupported operators error", {
  expect_error(
    plain_date(2020, 1, 1) + plain_date(2020, 1, 1),
    class = "vctrs_error_incompatible_op"
  )
  expect_error(plain_date(2020, 1, 1) * 2, class = "vctrs_error_incompatible_op")
  expect_error(plain_date(2020, 1, 1) - plain_time(1), class = "vctrs_error_incompatible_op")
  expect_error(temporal_add(1, duration(days = 1)), class = "zudate_type_error")
})
