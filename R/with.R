#' Replace fields
#'
#' `temporal_with()` returns a copy of `x` with some fields replaced, the
#' equivalent of Temporal's `.with()`. `with_plain_time()` and
#' `with_plain_date()` replace the whole time or date part of a plain
#' date-time (`.withPlainTime()`, `.withPlainDate()`).
#'
#' @param x A Temporal object.
#' @param ... Named fields to replace, recycled with `x` to a common length. Plain dates take
#'   `year`, `month`, `day`; plain times take `hour`, `minute`, `second`,
#'   `millisecond`, `microsecond`, `nanosecond`; plain date-times take both.
#' @param overflow How to handle out-of-range values: `"constrain"` (the
#'   default) clamps, `"reject"` raises an error.
#' @returns An object of the same class as `x`.
#' @export
#' @examples
#' d <- plain_date(2006, 1, 24)
#' temporal_with(d, day = 31, month = 2) # 2006-02-28
#' dt <- plain_date_time("1995-12-07T03:24:30")
#' temporal_with(dt, hour = 12)
#' with_plain_time(dt, plain_time(9, 30))
#' with_plain_time(dt) # midnight
#' with_plain_date(dt, plain_date(2000, 1, 1))
temporal_with <- function(x, ..., overflow = c("constrain", "reject")) {
  args <- rlang::list2(...)
  if (length(args) > 0 && (is.null(names(args)) || any(names(args) == ""))) {
    zudate_type_error("All fields in `...` must be named.")
  }
  if (is_plain_date(x)) {
    ctor <- plain_date
  } else if (is_plain_time(x)) {
    ctor <- plain_time
  } else if (is_plain_date_time(x)) {
    ctor <- plain_date_time
  } else {
    zudate_type_error(sprintf("`x` must be a Temporal object, not %s.", obj_type_friendly(x)))
  }
  n <- vec_size_common(x, !!!args)
  x <- vec_recycle(x, n)
  fields <- as.list(temporal_fields(x))
  unknown <- setdiff(names(args), names(fields))
  if (length(unknown) > 0) {
    zudate_type_error(sprintf(
      "Can't set unknown field%s %s.", if (length(unknown) > 1) "s" else "",
      paste0("`", unknown, "`", collapse = ", ")
    ))
  }
  args <- vec_recycle_common(!!!args, .size = n)
  fields[names(args)] <- args
  out <- do.call(ctor, c(fields, list(overflow = overflow)))
  # A missing element stays missing even when all its fields were replaced.
  vec_assign(out, vec_detect_missing(x), vec_init(out))
}

#' @rdname temporal_with
#' @param time A plain time (or string), recycled against `x`; `NULL` means
#'   midnight.
#' @export
with_plain_time <- function(x, time = NULL) {
  check_class(x, "zudate_plain_date_time", "a plain date-time")
  date <- to_plain_date(x)
  to_plain_date_time(date, time)
}

#' @rdname temporal_with
#' @param date A plain date (or string), recycled against `x`.
#' @export
with_plain_date <- function(x, date) {
  check_class(x, "zudate_plain_date_time", "a plain date-time")
  date <- as_plain_date(date)
  time <- to_plain_time(x)
  args <- vec_recycle_common(date = date, time = time)
  new_plain_date_time_from(args$date, args$time)
}
