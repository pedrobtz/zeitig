test_that("available_time_zones() lists IANA identifiers", {
  tz <- available_time_zones()
  expect_type(tz, "character")
  expect_gt(length(tz), 300)
  expect_false(is.unsorted(tz))
  expect_false(anyDuplicated(tz) > 0)
  expect_true(all(c("UTC", "America/New_York", "Europe/London", "Asia/Tokyo") %in% tz))
})

test_that("ZEITIG_TZDIR replaces the time zone database", {
  dir <- new_test_dir()
  dir.create(file.path(dir, "Test"))
  writeBin(tzif_fixed(3L * 3600L, "+03"), file.path(dir, "Test", "Plus3"))
  out <- zeitig_subprocess(
    c(
      'cat("zones:", available_time_zones(), "\\n")',
      'cat("offset:", offset(zoned_date_time(2020, 1, 1, time_zone = "Test/Plus3")), "\\n")',
      'cat("paris:", tryCatch("found", zeitig_error = function(e) "missing"), "\\n")'
    ),
    env = c(ZEITIG_TZDIR = dir)
  )
  expect_true("zones: Test/Plus3 " %in% out, info = paste(out, collapse = "\n"))
  expect_true("offset: +03:00 " %in% out, info = paste(out, collapse = "\n"))
})

test_that("an unreadable ZEITIG_TZDIR is an error, not a silent fallback", {
  dir <- new_test_dir()
  out <- zeitig_subprocess(
    c(
      "r <- tryCatch(available_time_zones(), zeitig_range_error = conditionMessage)",
      "cat('result:', r, '\\n')",
      "r <- tryCatch(zoned_date_time(2020, 1, 1, time_zone = 'UTC'),",
      "  zeitig_range_error = conditionMessage)",
      "cat('result:', r, '\\n')"
    ),
    env = c(ZEITIG_TZDIR = file.path(dir, "does-not-exist"))
  )
  hits <- grep("^result: ZEITIG_TZDIR", out, value = TRUE)
  expect_length(hits, 2)
})

test_that("UTC aliases count as one time zone for calendar-unit differences", {
  a <- zoned_date_time("2020-01-01T00:00Z[UTC]")
  b <- zoned_date_time("2020-01-03T12:00Z[Etc/UTC]")
  expect_equal(format(temporal_until(a, b, largest_unit = "day")), "P2DT12H")
  expect_equal(format(temporal_since(b, a, largest_unit = "day")), "P2DT12H")
  expect_true(temporal_equals(a, with_time_zone(a, "Etc/UTC")))
  paris <- zoned_date_time("2020-01-03T12:00+01:00[Europe/Paris]")
  expect_error(temporal_until(a, paris, largest_unit = "day"), class = "zeitig_range_error")
  expect_equal(format(temporal_until(a, paris)), "PT59H")
})
