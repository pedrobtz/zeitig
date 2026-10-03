test_that("plain_date_time() builds date-times from components", {
  x <- plain_date_time(1995, 12, 7, 3, 24, 30, 0, 3, 500)
  expect_s3_class(x, "zeitig_plain_date_time")
  expect_equal(format(x), "1995-12-07T03:24:30.0000035")
  expect_equal(format(plain_date_time(2020, 1, 1)), "2020-01-01T00:00:00")
  expect_equal(format(plain_date_time(2021, 2, 31, 25)), "2021-02-28T23:00:00")
  expect_error(plain_date_time(2021, 2, 31, overflow = "reject"), class = "zeitig_range_error")
  expect_error(plain_date_time(2021, 2, 1, 24, overflow = "reject"), class = "zeitig_range_error")
})

test_that("plain_date_time() parses strings", {
  x <- plain_date_time(c(
    "1995-12-07T03:24:30.0000035",
    "1995-12-07 03:24:30",
    "1995-12-07",
    "1995-12-07T03:24:30+01:00[Europe/Paris]",
    NA
  ))
  expect_equal(
    format(x),
    c(
      "1995-12-07T03:24:30.0000035", "1995-12-07T03:24:30", "1995-12-07T00:00:00",
      "1995-12-07T03:24:30", NA
    )
  )
  expect_error(plain_date_time("1995-12-07T03:24:30Z"), class = "zeitig_range_error")
})

test_that("fields", {
  x <- plain_date_time("2021-01-03T15:23:30.123456789")
  expect_equal(
    temporal_fields(x),
    data.frame(
      year = 2021L, month = 1L, day = 3L, hour = 15L, minute = 23L, second = 30L,
      millisecond = 123L, microsecond = 456L, nanosecond = 789L
    )
  )
  expect_equal(day_of_week(x), 7L)
  expect_equal(week_of_year(x), 53L)
})

test_that("NA and zero-length inputs", {
  x <- plain_date_time(c(2020, NA), 1, 1, c(NA, 1))
  expect_equal(is.na(x), c(TRUE, TRUE))
  expect_equal(format(x), c(NA_character_, NA_character_))
  expect_length(plain_date_time(integer(), integer(), integer()), 0)
  expect_length(plain_date_time(character()), 0)
})

test_that("conversions between civil types", {
  dt <- plain_date_time("1995-12-07T03:24:30.5")
  expect_equal(format(to_plain_date(dt)), "1995-12-07")
  expect_equal(format(to_plain_time(dt)), "03:24:30.5")
  expect_equal(
    format(to_plain_date_time(plain_date(2006, 8, 24), plain_time(15, 30))),
    "2006-08-24T15:30:00"
  )
  expect_equal(format(to_plain_date_time(plain_date(2006, 8, 24))), "2006-08-24T00:00:00")
  expect_equal(
    format(to_plain_date_time(plain_date(2006, 8, 24), c("01:00", "02:00"))),
    c("2006-08-24T01:00:00", "2006-08-24T02:00:00")
  )
  expect_equal(format(with_plain_time(dt, plain_time(9, 30))), "1995-12-07T09:30:00")
  expect_equal(format(with_plain_time(dt)), "1995-12-07T00:00:00")
  expect_equal(format(with_plain_date(dt, "2000-01-01")), "2000-01-01T03:24:30.5")
  expect_true(is.na(with_plain_date(dt, plain_date(NA, 1, 1))))
  expect_error(to_plain_date(plain_date(2000, 1, 1)), class = "zeitig_type_error")
  expect_error(to_plain_date_time(dt), class = "zeitig_type_error")
  expect_error(with_plain_time(plain_date(2000, 1, 1)), class = "zeitig_type_error")
})

test_that("temporal_with() on date-times", {
  dt <- plain_date_time("1995-12-07T03:24:30")
  expect_equal(format(temporal_with(dt, hour = 12, day = 31)), "1995-12-31T12:24:30")
})

test_that("ordering", {
  x <- plain_date_time(c("2020-01-01T12:00", "2020-01-01T08:00", "2019-12-31T23:59:59.999999999"))
  expect_equal(
    format(sort(x)),
    c("2019-12-31T23:59:59.999999999", "2020-01-01T08:00:00", "2020-01-01T12:00:00")
  )
  expect_equal(order(x), c(3L, 2L, 1L))
})
