test_that("instant() parses strings with offsets", {
  x <- instant(
    c("1969-07-20T20:17Z", "2020-01-01T12:00:00.5+01:00", "2020-01-01T00:00Z[Asia/Tokyo]", NA)
  )
  expect_s3_class(x, "zeitig_instant")
  expect_equal(
    format(x),
    c("1969-07-20T20:17:00Z", "2020-01-01T11:00:00.5Z", "2020-01-01T00:00:00Z", NA)
  )
  expect_error(instant("2020-01-01T00:00"), class = "zeitig_range_error")
  expect_error(instant("2020-01-01T00:00[Europe/Paris]"), class = "zeitig_range_error")
  expect_error(instant(1), class = "zeitig_type_error")
})

test_that("instant_from_epoch()", {
  expect_equal(format(instant_from_epoch(seconds = 0)), "1970-01-01T00:00:00Z")
  expect_equal(format(instant_from_epoch(seconds = -0.5)), "1969-12-31T23:59:59.5Z")
  expect_equal(format(instant_from_epoch(milliseconds = 1553906700000)), "2019-03-30T00:45:00Z")
  expect_equal(format(instant_from_epoch(milliseconds = -1)), "1969-12-31T23:59:59.999Z")
  expect_equal(
    format(instant_from_epoch(nanoseconds = c("1553906700000000001", "-1", NA))),
    c("2019-03-30T00:45:00.000000001Z", "1969-12-31T23:59:59.999999999Z", NA)
  )
  expect_equal(format(instant_from_epoch(nanoseconds = 1e9)), "1970-01-01T00:00:01Z")
  expect_error(instant_from_epoch(), class = "zeitig_type_error")
  expect_error(instant_from_epoch(seconds = 1, milliseconds = 1), class = "zeitig_type_error")
  expect_error(instant_from_epoch(seconds = 1e15), class = "zeitig_range_error")
  expect_error(instant_from_epoch(nanoseconds = "abc"), class = "zeitig_range_error")
  expect_error(instant_from_epoch(seconds = Inf), class = "zeitig_range_error")
})

test_that("epoch accessors", {
  x <- instant(c("2019-03-30T00:45:00.123456789Z", "1969-12-31T23:59:59.9Z", NA))
  expect_equal(epoch_seconds(x), c(1553906700, -1, NA))
  expect_equal(epoch_milliseconds(x), c(1553906700123, -100, NA))
  expect_equal(epoch_nanoseconds(x), c("1553906700123456789", "-100000000", NA))
  expect_error(epoch_seconds(plain_date(2020, 1, 1)), class = "zeitig_type_error")
})

test_that("arithmetic, difference and rounding", {
  x <- instant("2020-01-01T00:00Z")
  expect_equal(format(x + duration(hours = 1, nanoseconds = 1)), "2020-01-01T01:00:00.000000001Z")
  expect_equal(format(x - duration(minutes = 1)), "2019-12-31T23:59:00Z")
  expect_error(x + duration(days = 1), class = "zeitig_range_error")
  y <- instant("2020-01-02T01:30:00.5Z")
  expect_equal(format(y - x), "PT91800.5S")
  expect_equal(format(temporal_until(x, y, largest_unit = "hour")), "PT25H30M0.5S")
  expect_equal(format(temporal_since(x, y, largest_unit = "hour")), "-PT25H30M0.5S")
  expect_equal(
    format(temporal_until(
      x, y,
      largest_unit = "hour", smallest_unit = "minute", rounding_mode = "halfExpand"
    )),
    "PT25H30M"
  )
  expect_error(temporal_until(x, y, largest_unit = "day"), class = "zeitig_range_error")
  expect_equal(format(temporal_round(y, "hour")), "2020-01-02T02:00:00Z")
  expect_equal(format(temporal_round(y, "second", rounding_mode = "floor")), "2020-01-02T01:30:00Z")
  expect_equal(format(temporal_round(y, "minute", rounding_increment = 60)), "2020-01-02T02:00:00Z")
})

test_that("ordering, vctrs and NA", {
  x <- instant(c("2020-01-01T00:00Z", "1969-12-31T23:59:59.5Z", NA, "1970-01-01T00:00Z"))
  expect_equal(order(x), c(2L, 4L, 1L, 3L))
  expect_equal(is.na(x), c(FALSE, FALSE, TRUE, FALSE))
  expect_equal(temporal_compare(x, "1970-01-01T00:00Z"), c(1L, -1L, NA, 0L))
  expect_length(instant(character()), 0)
  expect_equal(format(c(x[1], x[2])), format(x)[1:2])
  expect_equal(data.frame(x = x)$x, x)
  expect_snapshot(instant(c("2020-01-01T00:00Z", NA)))
})

test_that("POSIXct interop", {
  ct <- as.POSIXct("2020-01-01 12:00:00.25", tz = "UTC")
  x <- as_instant(ct)
  expect_equal(format(x), "2020-01-01T12:00:00.25Z")
  expect_equal(as.POSIXct(x), ct)
  expect_equal(attr(as.POSIXct(x, tz = "Asia/Tokyo"), "tzone"), "Asia/Tokyo")
  expect_equal(as.POSIXlt(x)$hour, 12L)
  expect_equal(
    format(as_instant(as.POSIXct(-1.5, origin = "1970-01-01", tz = "UTC"))),
    "1969-12-31T23:59:58.5Z"
  )
  expect_true(is.na(as_instant(as.POSIXct(NA))))
  expect_equal(vec_cast(ct, instant(character())), x)
})
