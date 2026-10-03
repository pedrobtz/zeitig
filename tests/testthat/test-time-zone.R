test_that("available_time_zones() lists IANA identifiers", {
  tz <- available_time_zones()
  expect_type(tz, "character")
  expect_gt(length(tz), 300)
  expect_false(is.unsorted(tz))
  expect_false(anyDuplicated(tz) > 0)
  expect_true(all(c("UTC", "America/New_York", "Europe/London", "Asia/Tokyo") %in% tz))
})
