test_that("Date <-> plain_date is exact", {
  d <- as.Date(c("1969-12-31", "1970-01-01", "2020-02-29", "0001-01-01", NA))
  p <- as_plain_date(d)
  expect_equal(format(p), c("1969-12-31", "1970-01-01", "2020-02-29", "0001-01-01", NA))
  expect_equal(as.Date(p), d)
  expect_equal(vec_cast(d, plain_date(character())), p)
  expect_equal(vec_cast(p, as.Date(character())), d)
  expect_equal(as.Date(plain_date_time("2020-02-29T23:59")), as.Date("2020-02-29"))
  expect_equal(format(as_plain_date_time(as.Date("2020-02-29"))), "2020-02-29T00:00:00")
  # fractional Dates floor like base R
  expect_equal(format(as_plain_date(structure(-0.5, class = "Date"))), "1969-12-31")
})

test_that("POSIXct/POSIXlt -> plain types use the wall clock", {
  ct <- as.POSIXct("2020-02-29 12:30:15.25", tz = "UTC")
  expect_equal(format(as_plain_date_time(ct)), "2020-02-29T12:30:15.25")
  expect_equal(format(as_plain_date(ct)), "2020-02-29")
  expect_equal(format(as_plain_time(ct)), "12:30:15.25")
  ny <- as.POSIXct("2020-07-01 08:00:00", tz = "America/New_York")
  expect_equal(format(as_plain_date_time(ny)), "2020-07-01T08:00:00")
  lt <- as.POSIXlt("2020-02-29 12:30:15", tz = "Asia/Tokyo")
  expect_equal(format(as_plain_date_time(lt)), "2020-02-29T12:30:15")
  expect_true(is.na(as_plain_date_time(as.POSIXct(NA))))
  # microsecond rounding for POSIXct
  ct2 <- as.POSIXct(1e9 + 0.1234567, tz = "UTC", origin = "1970-01-01")
  expect_equal(format(as_plain_time(ct2)), "01:46:40.123457")
})

test_that("plain_date_time -> POSIXct/POSIXlt", {
  dt <- plain_date_time("2020-02-29T12:30:15.5")
  ct <- as.POSIXct(dt)
  expect_equal(ct, as.POSIXct("2020-02-29 12:30:15.5", tz = "UTC"))
  ny <- as.POSIXct(dt, tz = "America/New_York")
  expect_equal(attr(ny, "tzone"), "America/New_York")
  expect_equal(format(ny, "%H:%M:%S"), "12:30:15")
  lt <- as.POSIXlt(dt)
  expect_s3_class(lt, "POSIXlt")
  expect_equal(lt$hour, 12L)
  expect_equal(format(as_plain_date_time(as.POSIXct(dt))), format(dt))
})

test_that("as_*() reject unsupported inputs", {
  expect_error(as_plain_date(1), class = "zudate_type_error")
  expect_error(as_plain_time(TRUE), class = "zudate_type_error")
  expect_error(as_plain_date_time(list()), class = "zudate_type_error")
  expect_equal(format(as_plain_date(plain_date_time("2020-01-01T10:00"))), "2020-01-01")
  expect_equal(format(as_plain_time(plain_date_time("2020-01-01T10:00"))), "10:00:00")
  expect_equal(format(as_plain_date_time(plain_date(2020, 1, 1))), "2020-01-01T00:00:00")
})
