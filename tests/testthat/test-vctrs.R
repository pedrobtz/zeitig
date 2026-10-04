civil_examples <- function() {
  list(
    plain_date = plain_date(c("2020-01-01", "1999-12-31", NA)),
    plain_time = plain_time(c("12:00", "00:00:00.000000001", NA)),
    plain_date_time = plain_date_time(c("2020-01-01T12:00", "1999-12-31T23:59", NA))
  )
}

test_that("civil types behave like vectors", {
  for (x in civil_examples()) {
    expect_length(x, 3)
    expect_equal(is.na(x), c(FALSE, FALSE, TRUE))
    expect_equal(format(x[2:1]), format(x)[2:1])
    expect_equal(format(rev(x)), rev(format(x)))
    expect_equal(format(c(x, x)), rep(format(x), 2))
    expect_length(unique(c(x, x)), 3)
    expect_equal(format(x[[1]]), format(x)[1])
    expect_equal(as.character(x), format(x))
    expect_equal(vec_cast(format(x), x), x)
    df <- data.frame(x = x)
    expect_equal(nrow(df), 3)
    expect_equal(df$x, x)
    path <- tempfile(fileext = ".rds")
    saveRDS(x, path)
    expect_equal(readRDS(path), x)
    expect_equal(vec_size(vec_ptype(x)), 0)
    expect_equal(format(min(x, na.rm = TRUE)), sort(format(x))[1])
  }
})

test_that("printing", {
  expect_snapshot({
    plain_date(c("2020-01-01", NA))
    plain_time("12:30:00.5")
    plain_date_time("2020-01-01T12:30")
    data.frame(d = plain_date(2020, 1:2, 1), t = plain_time(1:2))
  })
})

test_that("types do not mix implicitly", {
  expect_error(c(plain_date(2020, 1, 1), plain_time(1)))
  expect_error(plain_date(2020, 1, 1) == plain_time(1))
})

test_that("assignment", {
  x <- plain_date(2020, 1:3, 1)
  x[2] <- plain_date(1999, 1, 1)
  expect_equal(format(x), c("2020-01-01", "1999-01-01", "2020-03-01"))
  x[3] <- "2000-02-02"
  expect_equal(format(x)[3], "2000-02-02")
})

test_that("strings combine with and compare against Temporal values", {
  d <- plain_date(2020, 1, 1:2)
  expect_equal(d == "2020-01-01", c(TRUE, FALSE))
  expect_equal("2020-01-02" == d, c(FALSE, TRUE))
  expect_equal(d < "2020-01-02", c(TRUE, FALSE))
  expect_equal(format(c(d, "2021-02-02")), c("2020-01-01", "2020-01-02", "2021-02-02"))
  expect_true(plain_time(12) == "12:00")
  expect_true(plain_date_time("2020-01-01T12:00") == "2020-01-01T12:00:00")
  expect_true(instant("2020-01-01T00:00Z") == "2020-01-01T01:00+01:00")
  z <- zoned_date_time("2020-01-01T00:00Z[UTC]")
  expect_true(z == "2020-01-01T01:00+01:00[Europe/Paris]")
  expect_false(duration(hours = 1) == "PT60M")
  expect_true(duration(minutes = 60) == "PT60M")
  expect_error(d == "not a date", class = "zeitig_range_error")
})
