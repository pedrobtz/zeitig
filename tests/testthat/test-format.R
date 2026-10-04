test_that("format() options on zoned date-times", {
  x <- zoned_date_time("2020-01-01T15:23:30.123456789+01:00[Europe/Paris]")
  expect_equal(format(x), "2020-01-01T15:23:30.123456789+01:00[Europe/Paris]")
  expect_equal(as.character(x), format(x))
  expect_equal(format(x, smallest_unit = "minute"), "2020-01-01T15:23+01:00[Europe/Paris]")
  expect_equal(format(x, smallest_unit = "seconds"), "2020-01-01T15:23:30+01:00[Europe/Paris]")
  expect_equal(
    format(x, fractional_second_digits = 2, rounding_mode = "halfExpand"),
    "2020-01-01T15:23:30.12+01:00[Europe/Paris]"
  )
  expect_equal(
    format(x, fractional_second_digits = 0, rounding_mode = "ceil"),
    "2020-01-01T15:23:31+01:00[Europe/Paris]"
  )
  expect_equal(
    format(x, offset = "never", time_zone_name = "critical", calendar_name = "always"),
    "2020-01-01T15:23:30.123456789[!Europe/Paris][u-ca=iso8601]"
  )
  expect_equal(
    format(x, time_zone_name = "never", calendar_name = "critical"),
    "2020-01-01T15:23:30.123456789+01:00[!u-ca=iso8601]"
  )
  expect_error(format(x, fractional_second_digits = 10), class = "zeitig_range_error")
  expect_error(format(x, smallest_unit = "hour"))
  expect_error(format(x, offset = "always"))
})

test_that("format() options on other types", {
  expect_equal(format(plain_time("12:00:00.5"), fractional_second_digits = 3), "12:00:00.500")
  expect_equal(format(plain_time("12:00:00.5"), smallest_unit = "minute"), "12:00")
  expect_equal(format(plain_date(2020, 1, 1), calendar_name = "always"), "2020-01-01[u-ca=iso8601]")
  expect_equal(format(plain_date(2020, 1, 1), calendar_name = "never"), "2020-01-01")
  expect_equal(
    format(
      plain_date_time("2020-01-01T23:59:59.9"),
      smallest_unit = "second", rounding_mode = "ceil"
    ),
    "2020-01-02T00:00:00"
  )
  expect_equal(
    format(plain_date_time("2020-01-01T12:00"), fractional_second_digits = 1),
    "2020-01-01T12:00:00.0"
  )
  i <- instant("2020-01-01T00:00:00.123Z")
  expect_equal(format(i, time_zone = "Asia/Tokyo"), "2020-01-01T09:00:00.123+09:00")
  expect_equal(
    format(i, smallest_unit = "second", time_zone = c("UTC")),
    "2020-01-01T00:00:00+00:00"
  )
  expect_equal(format(i, fractional_second_digits = 1), "2020-01-01T00:00:00.1Z")
  expect_equal(
    format(instant(c("2020-01-01T00:00Z", NA)), time_zone = "UTC"),
    c("2020-01-01T00:00:00+00:00", NA)
  )
})

test_that("temporal_strftime()", {
  expect_equal(temporal_strftime(plain_date(2024, 7, 15), "%A, %B %d, %Y"), "Monday, July 15, 2024")
  expect_equal(temporal_strftime(plain_time(16, 24, 59), "%I:%M %p"), "04:24 PM")
  expect_equal(
    temporal_strftime(
      zoned_date_time("2024-07-15T16:24:59-04:00[America/New_York]"),
      "%H:%M %Z (%z) %Q"
    ),
    "16:24 EDT (-0400) America/New_York"
  )
  expect_equal(temporal_strftime(instant("2024-07-15T16:24:59Z"), "%s"), "1721060699")
  expect_equal(
    temporal_strftime(plain_date_time("2024-07-15T16:24:59.5"), c("%Y", "%H:%M:%S%.f")),
    c("2024", "16:24:59.5")
  )
  expect_equal(temporal_strftime(plain_date(c(2024, NA), 7, 15), "%Y"), c("2024", NA))
  expect_error(temporal_strftime(plain_date(2024, 7, 15), "%H"), class = "zeitig_range_error")
  expect_error(temporal_strftime("2024-07-15", "%Y"), class = "zeitig_type_error")
})

test_that("temporal_strptime()", {
  expect_equal(format(temporal_strptime("15/07/2024", "%d/%m/%Y", "plain_date")), "2024-07-15")
  expect_equal(format(temporal_strptime("4:30 PM", "%I:%M %p", "plain_time")), "16:30:00")
  expect_equal(
    format(temporal_strptime("2024-07-15 16:24:01.25", "%Y-%m-%d %H:%M:%S%.f", "plain_date_time")),
    "2024-07-15T16:24:01.25"
  )
  expect_equal(
    format(temporal_strptime("2024-07-15 16:24 -0400", "%Y-%m-%d %H:%M %z", "instant")),
    "2024-07-15T20:24:00Z"
  )
  expect_equal(
    format(temporal_strptime(
      "2024-07-15 16:24 America/New_York", "%Y-%m-%d %H:%M %Q", "zoned_date_time"
    )),
    "2024-07-15T16:24:00-04:00[America/New_York]"
  )
  expect_equal(
    format(temporal_strptime(c("2024-03-05", NA), "%Y-%m-%d", "plain_date")),
    c("2024-03-05", NA)
  )
  expect_error(
    temporal_strptime("2024-07-15", "%d/%m/%Y", "plain_date"),
    class = "zeitig_range_error"
  )
  expect_error(
    temporal_strptime("2024-07-15 16:24", "%Y-%m-%d %H:%M", "instant"),
    class = "zeitig_range_error"
  )
  expect_error(temporal_strptime("x", "%Y", "duration"))
})

test_that("summary() and str()", {
  s <- summary(plain_date(2020, 1:12, 1))
  expect_s3_class(s, "table")
  expect_equal(names(s), c("Min.", "1st Qu.", "Median", "3rd Qu.", "Max."))
  expect_equal(unclass(unname(s[c(1, 3, 5)])), c("2020-01-01", "2020-06-01", "2020-12-01"))
  s <- summary(duration(hours = c(5, 1, NA)))
  expect_equal(unclass(unname(s[c("Min.", "Max.", "NA's")])), c("PT1H", "PT5H", "1"))
  expect_equal(unclass(unname(summary(plain_time(character()))[1])), NA_character_)
  expect_snapshot({
    summary(zoned_date_time(2020, 1, 1:3, time_zone = "UTC"))
    str(plain_date(2020, 1:3, 1))
  })
})

test_that("format() rounds in the same pass as it prints", {
  z <- zoned_date_time(c(
    "2020-01-01T15:23:30.9996+01:00[Europe/Paris]", "2020-01-01T23:59:59.9999+01:00[Europe/Paris]"
  ))
  expect_equal(
    format(z, fractional_second_digits = 3, rounding_mode = "halfExpand"),
    c("2020-01-01T15:23:31.000+01:00[Europe/Paris]", "2020-01-02T00:00:00.000+01:00[Europe/Paris]")
  )
  expect_equal(
    format(z, smallest_unit = "minute", rounding_mode = "halfEven", time_zone_name = "never"),
    c("2020-01-01T15:24+01:00", "2020-01-02T00:00+01:00")
  )
  i <- instant(c("2020-01-01T00:00:00.5Z", "1969-12-31T23:59:59.5Z"))
  expect_equal(
    format(
      i,
      smallest_unit = "second", rounding_mode = "trunc", time_zone = c("Asia/Tokyo", "UTC")
    ),
    c("2020-01-01T09:00:00+09:00", "1969-12-31T23:59:59+00:00")
  )
  expect_error(
    format(plain_time("12:00"), smallest_unit = "minute", rounding_mode = "bogus"),
    class = "zeitig_range_error"
  )
  expect_error(temporal_strftime(plain_date(2020, 1:3, 1), c("%Y", "%m")))
  expect_equal(temporal_strftime(plain_date(2020, 1:2, 1), c("%Y", "%m")), c("2020", "02"))
})
