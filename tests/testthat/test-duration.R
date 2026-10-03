test_that("duration() builds durations from fields", {
  d <- duration(hours = 1, minutes = 30)
  expect_s3_class(d, "zietig_duration")
  expect_equal(format(d), "PT1H30M")
  expect_equal(format(duration()), "PT0S")
  expect_equal(format(duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)), "P1Y2M3W4DT5H6M7.00800901S")
  expect_equal(format(duration(milliseconds = 1500)), "PT1.5S")
  expect_equal(format(duration(minutes = 90)), "PT90M")
  expect_equal(format(duration(days = -3, hours = -1)), "-P3DT1H")
  expect_equal(format(duration(hours = 1:3)), c("PT1H", "PT2H", "PT3H"))
})

test_that("field validation follows Temporal", {
  expect_error(duration(hours = 1, minutes = -1), "mixed-sign", class = "zietig_range_error")
  expect_error(duration(hours = 1.5), class = "zietig_range_error")
  expect_error(duration(hours = Inf), class = "zietig_range_error")
  expect_error(duration(years = 20000), class = "zietig_range_error")
  expect_error(duration(hours = "1"), class = "zietig_type_error")
  expect_error(duration(days = c(1, 2, -1), hours = 1), "element 3")
})

test_that("duration() parses ISO 8601 strings", {
  x <- duration(c("P1Y2M3DT4H", "-PT1.5S", "PT0S", "p1d", "PT36H", NA))
  expect_equal(format(x), c("P1Y2M3DT4H", "-PT1.5S", "PT0S", "P1D", "PT36H", NA))
  expect_error(duration("1 day"), class = "zietig_range_error")
  expect_error(duration("P"), class = "zietig_range_error")
  expect_equal(
    temporal_fields(duration("-PT1.5S")),
    data.frame(
      years = 0, months = 0, weeks = 0, days = 0, hours = 0, minutes = 0,
      seconds = -1, milliseconds = -500, microseconds = 0, nanoseconds = 0
    )
  )
})

test_that("NA and zero-length", {
  x <- duration(hours = c(1, NA))
  expect_equal(is.na(x), c(FALSE, TRUE))
  expect_equal(format(x), c("PT1H", NA))
  expect_length(duration(hours = numeric()), 0)
  expect_length(duration(character()), 0)
  expect_equal(sign(x), c(1L, NA))
  expect_equal(duration_blank(x), c(FALSE, NA))
  expect_equal(duration_total(x, "minute"), c(60, NA))
  expect_equal(format(-x), c("-PT1H", NA))
})

test_that("negation, abs, sign, blank", {
  d <- duration(c("-P1D", "PT0S", "PT1S"))
  expect_equal(format(-d), c("P1D", "PT0S", "-PT1S"))
  expect_equal(format(+d), format(d))
  expect_equal(format(abs(d)), c("P1D", "PT0S", "PT1S"))
  expect_equal(sign(d), c(-1L, 0L, 1L))
  expect_equal(duration_blank(d), c(FALSE, TRUE, FALSE))
  expect_error(sqrt(d), class = "zietig_type_error")
})

test_that("adding durations balances with 24-hour days", {
  expect_equal(format(duration(hours = 1) + duration(minutes = 90)), "PT2H30M")
  expect_equal(format(duration(days = 1) - duration(hours = 1)), "PT23H")
  expect_equal(format(duration(hours = 12) + duration(hours = 13)), "PT25H")
  expect_error(duration(months = 1) + duration(days = 1), class = "zietig_range_error")
  expect_error(duration(days = 1) * 2, class = "vctrs_error_incompatible_op")
})

test_that("ordering uses the length with 24-hour days", {
  x <- duration(c("PT2H", "PT90M", "P1D", "-PT1S"))
  expect_equal(format(sort(x)), c("-PT1S", "PT90M", "PT2H", "P1D"))
  expect_true(duration(hours = 2) > duration(minutes = 90))
  # == compares fields
  expect_false(duration(hours = 1) == duration(minutes = 60))
  expect_error(sort(duration(c("P1M", "P1D"))), class = "zietig_range_error")
})

test_that("duration_total() and duration_compare()", {
  expect_equal(duration_total(duration(hours = 1, minutes = 30), "minute"), 90)
  expect_equal(duration_total(duration(hours = 1, minutes = 30), "hours"), 1.5)
  expect_equal(duration_total(duration(days = 1), "hour"), 24)
  expect_equal(
    duration_total(duration(months = 1), "day", relative_to = plain_date(2020, 2, 1)),
    29
  )
  expect_equal(
    duration_total(duration(months = 1), "day", relative_to = c("2020-01-01", "2020-02-01")),
    c(31, 29)
  )
  expect_error(duration_total(duration(months = 1), "day"), class = "zietig_range_error")
  expect_equal(duration_compare(duration(hours = 1), duration(minutes = 60)), 0L)
  expect_equal(duration_compare(duration(hours = 1), duration(minutes = c(59, 61))), c(1L, -1L))
  expect_equal(
    duration_compare(duration(months = 1), duration(days = 30), relative_to = "2020-02-01"),
    -1L
  )
  expect_equal(
    duration_compare(duration(months = 1), duration(days = 30), relative_to = "2020-01-01"),
    1L
  )
  expect_equal(duration_compare("PT1H", NA_character_), NA_integer_)
  expect_error(duration_total(duration(hours = 1), "fortnight"))
  expect_error(
    duration_total(duration(hours = 1), "day", relative_to = 1),
    class = "zietig_type_error"
  )
})

test_that("temporal_round() on durations", {
  expect_equal(format(temporal_round(duration(minutes = 130), largest_unit = "hour")), "PT2H10M")
  expect_equal(
    format(temporal_round(duration(hours = 1, minutes = 29), "hour")),
    "PT1H"
  )
  expect_equal(
    format(temporal_round(duration(hours = 1, minutes = 30), "hour")),
    "PT2H"
  )
  expect_equal(
    format(temporal_round(duration(hours = 1, minutes = 30), "hour", rounding_mode = "halfEven")),
    "PT2H"
  )
  expect_equal(
    format(temporal_round(duration(minutes = 7), "minute", rounding_increment = 5)),
    "PT5M"
  )
  expect_equal(
    format(temporal_round(duration(days = 45), largest_unit = "month", relative_to = "2020-01-01")),
    "P1M14D"
  )
  expect_error(
    temporal_round(duration(days = 45), largest_unit = "month"),
    class = "zietig_range_error"
  )
  expect_error(temporal_round(duration(hours = 1)), class = "zietig_type_error")
  expect_error(temporal_round(duration(hours = 1), "hour", rounding_mode = "up"))
})

test_that("difftime conversions", {
  expect_equal(format(as_duration(as.difftime(90, units = "mins"))), "PT90M")
  expect_equal(format(as_duration(as.difftime(2, units = "days"))), "P2D")
  expect_equal(format(as_duration(as.difftime(1.5, units = "hours"))), "PT5400S")
  expect_equal(format(as_duration(as.difftime(-1.000001, units = "secs"))), "-PT1.000001S")
  expect_equal(format(as_duration(as.difftime(1, units = "weeks"))), "PT604800S")
  expect_equal(format(as_duration(as.difftime(c(1, NA), units = "secs"))), c("PT1S", NA))
  dt <- as_difftime(duration(hours = 1, minutes = 30))
  expect_s3_class(dt, "difftime")
  expect_equal(units(dt), "hours")
  expect_equal(as.double(dt), 1.5)
  expect_equal(as.double(as_difftime(duration(hours = 36), units = "days")), 1.5)
  expect_equal(units(as_difftime(duration(seconds = 5))), "secs")
  expect_error(as_difftime(duration(months = 1)), class = "zietig_range_error")
  expect_error(as_duration(1), class = "zietig_type_error")
})

test_that("vctrs behaviour", {
  x <- duration(c("P1D", "PT1S", NA))
  expect_equal(format(c(x, x)), rep(format(x), 2))
  expect_equal(format(rev(x)), rev(format(x)))
  expect_equal(vec_cast(format(x), x), x)
  expect_equal(data.frame(x = x)$x, x)
  path <- tempfile(fileext = ".rds")
  saveRDS(x, path)
  expect_equal(readRDS(path), x)
  expect_snapshot(duration(c("P1Y2M", "-PT1.5S", NA)))
})
