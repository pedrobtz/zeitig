#' Compare Temporal objects
#'
#' Temporal objects support the usual comparison operators (`<`, `==`, ...),
#' `sort()`, `order()`, `min()`/`max()` and `unique()`, all with Temporal's
#' `compare()` ordering. `temporal_compare()` is the equivalent of
#' `Temporal.X.compare(a, b)` and `temporal_equals()` of `a.equals(b)`.
#'
#' Zoned date-times compare by their exact time only, so `==` is `TRUE` for
#' the same instant in two time zones; `temporal_equals()` additionally
#' requires the same time zone, as Temporal's `equals()`.
#'
#' Character vectors are parsed as the type of the other argument.
#'
#' @param x,y Temporal objects of the same type (or strings), recycled to a
#'   common length.
#' @returns `temporal_compare()`: an integer vector of -1, 0 and 1.
#'   `temporal_equals()`: a logical vector. Both are `NA` where either input is
#'   missing.
#' @export
#' @examples
#' a <- plain_date(c("2020-01-01", "2021-06-30"))
#' temporal_compare(a, "2021-01-01")
#' temporal_equals(a, "2020-01-01")
#' sort(plain_time(c("12:00", "08:30", "23:59:59.5")))
temporal_compare <- function(x, y) {
  args <- temporal_common(x, y)
  vec_compare(args[[1]], args[[2]])
}

#' @rdname temporal_compare
#' @export
temporal_equals <- function(x, y) {
  args <- temporal_common(x, y)
  out <- vec_equal(args[[1]], args[[2]])
  if (is_zoned_date_time(args[[1]])) {
    out <- out & time_zone(args[[1]]) == time_zone(args[[2]])
  }
  out
}

temporal_common <- function(x, y, call = rlang::caller_env()) {
  if (is.character(x) && !is.character(y)) x <- vec_cast(x, y, call = call)
  if (is.character(y) && !is.character(x)) y <- vec_cast(y, x, call = call)
  args <- vec_cast_common(x = x, y = y, .call = call)
  vec_recycle_common(!!!args, .call = call)
}
