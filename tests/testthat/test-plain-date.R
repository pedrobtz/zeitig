test_that("plain_date() builds dates from components", {
  d <- plain_date(2006, 8, 24)
  expect_s3_class(d, "zietig_plain_date")
  expect_equal(format(d), "2006-08-24")
  expect_equal(format(plain_date(2020L, 1:3, 31L)), c("2020-01-31", "2020-02-29", "2020-03-31"))
  # fractions truncate towards zero (ToIntegerWithTruncation)
  expect_equal(format(plain_date(2006.9, 8.5, 24.999)), "2006-08-24")
})

test_that("overflow = 'constrain' clamps, 'reject' errors", {
  expect_equal(format(plain_date(2021, 2, 31)), "2021-02-28")
  expect_equal(format(plain_date(2021, 13, 1)), "2021-12-01")
  expect_error(plain_date(2021, 2, 31, overflow = "reject"), "1..=28", class = "zietig_range_error")
  expect_error(plain_date(2021, 13, 1, overflow = "reject"), class = "zietig_range_error")
  # non-positive months and days are always an error
  expect_error(plain_date(2021, 0, 1), class = "zietig_range_error")
  expect_error(plain_date(2021, 1, 0), class = "zietig_range_error")
  expect_error(plain_date(10000, 1, 1), class = "zietig_range_error")
  expect_error(plain_date(2021, 1, 1, overflow = "nope"))
})

test_that("errors report the failing element", {
  expect_error(plain_date(2021, 1, c(1, 2, 40), overflow = "reject"), "element 3")
  expect_error(plain_date(c("2020-01-01", "2020-13-01")), "element 2", class = "zietig_range_error")
})

test_that("bad argument types are type errors", {
  expect_error(plain_date(2021, "a", 1), class = "zietig_type_error")
  expect_error(plain_date(2021, Inf, 1), class = "zietig_range_error")
  expect_error(plain_date(1, 1, 1, 1))
})

test_that("plain_date() parses RFC 9557 strings", {
  x <- plain_date(c(
    "2006-08-24",
    "20060824",
    "2006-08-24T15:43:27",
    "2019-11-18T15:23:30.123+01:00[Europe/Paris]",
    "-000001-01-01",
    NA
  ))
  expect_equal(
    format(x),
    c("2006-08-24", "2006-08-24", "2006-08-24", "2019-11-18", "-000001-01-01", NA)
  )
  # UTC designator is not allowed for plain types
  expect_error(plain_date("2006-08-24T15:43:27Z"), class = "zietig_range_error")
  expect_error(plain_date("not a date"), class = "zietig_range_error")
  expect_error(plain_date(c("2006-08-24", "2006-02-30")), "element 2")
})

test_that("NA and zero-length inputs", {
  x <- plain_date(c(2020, NA), 1, 1)
  expect_equal(is.na(x), c(FALSE, TRUE))
  expect_equal(format(x), c("2020-01-01", NA))
  expect_equal(year(x), c(2020L, NA))
  expect_equal(day_of_week(x), c(3L, NA))
  expect_length(plain_date(integer(), integer(), integer()), 0)
  expect_length(plain_date(character()), 0)
  expect_equal(format(plain_date(character())), character())
  expect_equal(year(plain_date(character())), integer())
})

test_that("recycling follows vctrs rules", {
  expect_length(plain_date(2020, 1:12, 1), 12)
  expect_error(plain_date(2020, 1:2, 1:3))
})

test_that("date fields match Temporal", {
  d <- plain_date(2006, 8, 24)
  expect_equal(year(d), 2006L)
  expect_equal(month(d), 8L)
  expect_equal(day(d), 24L)
  expect_equal(day_of_week(d), 4L) # Thursday
  expect_equal(day_of_year(d), 236L)
  expect_equal(week_of_year(d), 34L)
  expect_equal(year_of_week(d), 2006L)
  expect_equal(days_in_week(d), 7L)
  expect_equal(days_in_month(d), 31L)
  expect_equal(days_in_year(d), 365L)
  expect_equal(months_in_year(d), 12L)
  expect_false(in_leap_year(d))

  # ISO week-numbering year differs around New Year
  x <- plain_date(c("2021-01-03", "2019-12-30", "2020-12-31"))
  expect_equal(week_of_year(x), c(53L, 1L, 53L))
  expect_equal(year_of_week(x), c(2020L, 2020L, 2020L))
  expect_equal(in_leap_year(plain_date(c(1900, 2000, 2024), 1, 1)), c(FALSE, TRUE, TRUE))
  expect_equal(days_in_month(plain_date(2024, 2, 1)), 29L)
})

test_that("time fields are not available on dates", {
  expect_error(hour(plain_date(2020, 1, 1)), class = "zietig_type_error")
  expect_error(year(1), class = "zietig_type_error")
})

test_that("temporal_with() replaces fields", {
  d <- plain_date(2006, 1, 24)
  expect_equal(format(temporal_with(d, day = 31, month = 2)), "2006-02-28")
  expect_error(
    temporal_with(d, day = 31, month = 2, overflow = "reject"),
    class = "zietig_range_error"
  )
  expect_equal(
    format(temporal_with(d, year = 2000:2002)),
    c("2000-01-24", "2001-01-24", "2002-01-24")
  )
  expect_error(temporal_with(d, hour = 1), "unknown field", class = "zietig_type_error")
  expect_error(temporal_with(d, 1), class = "zietig_type_error")
  # a missing element stays missing
  expect_true(is.na(temporal_with(plain_date(NA, 1, 1), year = 2000)))
})

test_that("comparison follows Temporal.PlainDate.compare", {
  a <- plain_date(c("2020-01-01", "2021-06-30", NA))
  expect_equal(temporal_compare(a, "2021-01-01"), c(-1L, 1L, NA))
  expect_equal(temporal_equals(a, plain_date(2020, 1, 1)), c(TRUE, FALSE, NA))
  expect_true(plain_date(2020, 1, 1) < plain_date(2020, 1, 2))
  expect_true(plain_date(2019, 12, 31) < plain_date(2020, 1, 1))
  expect_equal(
    format(sort(plain_date(c("2020-03-01", "-000005-01-01", "2020-02-29")))),
    c("-000005-01-01", "2020-02-29", "2020-03-01")
  )
})
