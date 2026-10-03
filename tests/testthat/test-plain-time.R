test_that("plain_time() builds times from components", {
  t <- plain_time(19, 39, 9, 68, 346, 205)
  expect_s3_class(t, "zietig_plain_time")
  expect_equal(format(t), "19:39:09.068346205")
  expect_equal(format(plain_time()), "00:00:00")
  expect_equal(format(plain_time(15, 23, 30, 500)), "15:23:30.5")
  expect_equal(format(plain_time(15, 23, 30, 0, 1)), "15:23:30.000001")
})

test_that("overflow handling", {
  expect_equal(format(plain_time(25, 61, 61, 1000, 1000, 1000)), "23:59:59.999999999")
  expect_equal(format(plain_time(-1)), "00:00:00")
  expect_error(plain_time(24, overflow = "reject"), "hour", class = "zietig_range_error")
  expect_error(plain_time(0, 60, overflow = "reject"), "minute", class = "zietig_range_error")
  expect_error(plain_time(0, 0, 0, 0, 0, c(1, 1000), overflow = "reject"), "element 2")
})

test_that("plain_time() parses strings", {
  x <- plain_time(c("03:24:30", "15:23", "152330", "15:23:30.123456789", "2019-11-18T15:23:30", NA))
  expect_equal(
    format(x),
    c("03:24:30", "15:23:00", "15:23:30", "15:23:30.123456789", "15:23:30", NA)
  )
  expect_error(plain_time("25:00"), class = "zietig_range_error")
  expect_error(plain_time("2019-11-18T15:23:30Z"), class = "zietig_range_error")
})

test_that("time fields", {
  t <- plain_time(19, 39, 9, 68, 346, 205)
  expect_equal(hour(t), 19L)
  expect_equal(minute(t), 39L)
  expect_equal(second(t), 9L)
  expect_equal(millisecond(t), 68L)
  expect_equal(microsecond(t), 346L)
  expect_equal(nanosecond(t), 205L)
  expect_error(year(t), class = "zietig_type_error")
  expect_equal(
    temporal_fields(t),
    data.frame(
      hour = 19L, minute = 39L, second = 9L, millisecond = 68L,
      microsecond = 346L, nanosecond = 205L
    )
  )
})

test_that("NA and zero-length inputs", {
  x <- plain_time(c(1, NA))
  expect_equal(format(x), c("01:00:00", NA))
  expect_equal(hour(x), c(1L, NA))
  expect_equal(is.na(x), c(FALSE, TRUE))
  expect_length(plain_time(integer()), 0)
  expect_length(plain_time(character()), 0)
  expect_equal(nanosecond(plain_time(character())), integer())
})

test_that("temporal_with() and ordering", {
  t <- plain_time(19, 39, 9)
  expect_equal(format(temporal_with(t, minute = 0, second = 0)), "19:00:00")
  expect_equal(format(temporal_with(t, hour = 30)), "23:39:09")
  expect_error(temporal_with(t, hour = 30, overflow = "reject"), class = "zietig_range_error")
  x <- plain_time(c("12:00", "08:30", "23:59:59.5", "23:59:59.25"))
  expect_equal(format(sort(x)), c("08:30:00", "12:00:00", "23:59:59.25", "23:59:59.5"))
  expect_equal(temporal_compare(x, "12:00"), c(0L, -1L, 1L, 1L))
})
