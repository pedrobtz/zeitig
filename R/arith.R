#' Add or subtract durations
#'
#' `temporal_add()` and `temporal_subtract()` are Temporal's `.add()` and
#' `.subtract()`. The `+` and `-` operators do the same with the default
#' `overflow = "constrain"`.
#'
#' * Plain dates: years and months are added first, clamping the day to the
#'   end of the month (`2021-01-31` + 1 month is `2021-02-28`), then weeks
#'   and days; time units are balanced into whole 24-hour days and any
#'   remainder is ignored.
#' * Plain times: only hours and smaller units count and the result wraps
#'   around midnight.
#' * Plain date-times: calendar units as for dates, then the time.
#' * Instants: only hours and smaller units are allowed (exact time).
#' * Zoned date-times: calendar units (years to days) follow the wall clock
#'   and are DST-aware (adding one day keeps the local time even when the day
#'   has 23 or 25 hours); time units are exact elapsed time.
#'
#' Subtracting two values of the same type (`x - y`) is `temporal_since(x, y)`
#' with default options, giving a duration.
#'
#' @param x A plain date, plain time, plain date-time, instant or zoned
#'   date-time.
#' @param duration A duration (or ISO 8601 duration string), recycled against
#'   `x`.
#' @param ... These dots are for future extensions and must be empty.
#' @param overflow `"constrain"` (default) clamps the day of month when
#'   adding years or months; `"reject"` raises an error instead.
#' @returns An object of the same class as `x`.
#' @family arithmetic
#' @export
#' @examples
#' d <- plain_date(2021, 1, 31)
#' temporal_add(d, duration(months = 1))
#' try(temporal_add(d, "P1M", overflow = "reject"))
#' d + duration(days = 1:3)
#' plain_time(23, 30) + duration(hours = 1)
#' plain_date_time("2020-02-29T12:00") - duration(years = 1)
temporal_add <- function(x, duration, ..., overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  temporal_add_impl(x, as_duration(duration), arg_overflow(overflow))
}

#' @rdname temporal_add
#' @export
temporal_subtract <- function(x, duration, ..., overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  temporal_add_impl(x, -as_duration(duration), arg_overflow(overflow))
}

temporal_add_impl <- function(x, d, reject, call = rlang::caller_env()) {
  args <- vec_recycle_common(x = x, d = d, .call = call)
  x <- args$x
  d <- duration_data(args$d)
  if (is_plain_date(x)) {
    f <- vec_data(x)
    new_plain_date_fields(zietig_call(
      rs_plain_date_add(f$year, f$month, f$day, d, reject),
      call = call
    ))
  } else if (is_plain_time(x)) {
    f <- vec_data(x)
    new_plain_time_fields(zietig_call(
      rs_plain_time_add(f$second_of_day, f$nanos, d),
      call = call
    ))
  } else if (is_plain_date_time(x)) {
    new_plain_date_time_fields(zietig_call(
      rs_plain_date_time_add(unclass(vec_data(x)), d, reject),
      call = call
    ))
  } else if (is_instant(x)) {
    new_instant_fields(zietig_call(rs_instant_add(instant_data(x), d), call = call))
  } else if (is_zoned_date_time(x)) {
    new_zoned_fields(zietig_call(rs_zoned_add(zoned_data(x), d, reject), call = call))
  } else {
    zietig_type_error(
      sprintf("Can't add a duration to %s.", obj_type_friendly(x)),
      call = call
    )
  }
}

#' Difference between two values
#'
#' `temporal_until(x, y)` is the duration from `x` to `y` (Temporal's
#' `x.until(y)`); `temporal_since(x, y)` is the duration from `y` to `x`
#' (`x.since(y)`), i.e. the negation of `temporal_until(x, y)`.
#'
#' The result is balanced up to `largest_unit` and rounded to
#' `smallest_unit`. The default largest unit (`"auto"`) is `"day"` for plain
#' dates and plain date-times, `"hour"` for plain times and zoned
#' date-times, and `"second"` for instants; the default smallest unit is
#' `"day"` for plain dates and `"nanosecond"` otherwise. For zoned
#' date-times, calendar units (`largest_unit` of `"day"` or more) require both
#' values to have the same time zone.
#'
#' @param x,y Values of the same type (or strings parsed as that type),
#'   recycled to a common length.
#' @param ... These dots are for future extensions and must be empty.
#' @param largest_unit,smallest_unit Units such as `"year"`, `"day"` or
#'   `"millisecond"` (plurals accepted); `largest_unit` may be `"auto"` and
#'   `smallest_unit` `NULL` for the type's default.
#' @param rounding_increment Round to multiples of this many
#'   `smallest_unit`s.
#' @param rounding_mode One of `"trunc"` (default), `"ceil"`, `"floor"`,
#'   `"expand"`, `"halfCeil"`, `"halfFloor"`, `"halfExpand"`, `"halfTrunc"`,
#'   `"halfEven"`.
#' @returns A duration vector.
#' @family arithmetic
#' @export
#' @examples
#' temporal_until(plain_date(2006, 8, 24), plain_date(2019, 1, 31))
#' temporal_until("2006-08-24", plain_date(2019, 1, 31), largest_unit = "year")
#' temporal_since(plain_time(19, 39), plain_time(9, 0))
#' plain_date(2019, 1, 31) - plain_date(2006, 8, 24)
#' temporal_until(
#'   plain_date_time("2020-01-01T00:00"), plain_date_time("2020-01-02T13:31"),
#'   smallest_unit = "hour", rounding_mode = "halfExpand"
#' )
temporal_until <- function(x, y, ..., largest_unit = "auto", smallest_unit = NULL,
                           rounding_increment = 1, rounding_mode = "trunc") {
  rlang::check_dots_empty0(...)
  temporal_diff(x, y, largest_unit, smallest_unit, rounding_increment, rounding_mode, FALSE)
}

#' @rdname temporal_until
#' @export
temporal_since <- function(x, y, ..., largest_unit = "auto", smallest_unit = NULL,
                           rounding_increment = 1, rounding_mode = "trunc") {
  rlang::check_dots_empty0(...)
  temporal_diff(x, y, largest_unit, smallest_unit, rounding_increment, rounding_mode, TRUE)
}

temporal_diff <- function(x, y, largest_unit, smallest_unit, rounding_increment, rounding_mode,
                          since, call = rlang::caller_env()) {
  largest_unit <- arg_unit(largest_unit, auto = TRUE, call = call)
  rounding_increment <- arg_increment(rounding_increment, call = call)
  rounding_mode <- arg_rounding_mode(rounding_mode, call = call)
  args <- temporal_common(x, y, call = call)
  x <- args[[1]]
  y <- args[[2]]
  smallest_unit <- smallest_unit %||% if (is_plain_date(x)) "day" else "nanosecond"
  smallest_unit <- arg_unit(smallest_unit, call = call)
  fn <- if (is_plain_date(x)) {
    rs_plain_date_diff
  } else if (is_plain_time(x)) {
    rs_plain_time_diff
  } else if (is_plain_date_time(x)) {
    rs_plain_date_time_diff
  } else if (is_instant(x)) {
    rs_instant_diff
  } else if (is_zoned_date_time(x)) {
    rs_zoned_diff
  } else {
    zietig_type_error(
      sprintf("Can't compute a difference between %s.", obj_type_friendly(x)),
      call = call
    )
  }
  new_duration_fields(zietig_call(
    fn(
      unclass(vec_data(x)), unclass(vec_data(y)), largest_unit, smallest_unit,
      rounding_increment, rounding_mode, since
    ),
    call = call
  ))
}

#' Round values and durations
#'
#' `temporal_round()` is Temporal's `.round()`.
#'
#' * Plain times, plain date-times, instants and zoned date-times round to
#'   `smallest_unit` (at most `"hour"` for times and instants, `"day"` for
#'   date-times; a zoned day may be 23 or 25 hours long).
#'   `rounding_increment` must divide evenly into the next larger unit (for
#'   instants, into a day).
#' * Durations are rounded and balanced between `largest_unit` and
#'   `smallest_unit`. Without `relative_to`, days are 24 hours and years,
#'   months and weeks are an error; with it, calendar units are resolved from
#'   that date.
#'
#' @param x A plain time, plain date-time, instant, zoned date-time or
#'   duration.
#' @param smallest_unit The unit to round to (plurals accepted). For durations
#'   the default is `"nanosecond"`.
#' @param ... These dots are for future extensions and must be empty.
#' @param rounding_increment Round to multiples of this many
#'   `smallest_unit`s.
#' @param rounding_mode Defaults to `"halfExpand"` (round half away from
#'   zero); see [temporal_until()] for the other modes.
#' @param largest_unit Durations only: the largest unit of the result
#'   (`"auto"` keeps the largest non-zero unit of `x`).
#' @param relative_to Durations only: `NULL`, or a plain date, plain
#'   date-time or zoned date-time (or string) recycled against `x`.
#' @returns An object of the same class as `x`.
#' @family arithmetic
#' @export
#' @examples
#' temporal_round(plain_time(19, 39, 9, 68, 346, 205), "hour")
#' temporal_round(plain_time(19, 39, 9), "minute", rounding_increment = 15, rounding_mode = "floor")
#' temporal_round(plain_date_time("1995-12-07T03:24:30.000003500"), "second")
#' temporal_round(duration(minutes = 130), largest_unit = "hour")
#' temporal_round(duration(days = 45), largest_unit = "month", relative_to = "2020-01-01")
temporal_round <- function(x, smallest_unit = NULL, ..., rounding_increment = 1,
                           rounding_mode = "halfExpand", largest_unit = NULL,
                           relative_to = NULL) {
  rlang::check_dots_empty0(...)
  rounding_increment <- arg_increment(rounding_increment)
  rounding_mode <- arg_rounding_mode(rounding_mode)
  if (is_duration(x)) {
    if (is.null(smallest_unit) && is.null(largest_unit)) {
      zietig_type_error("At least one of `smallest_unit` and `largest_unit` must be given.")
    }
    smallest_unit <- arg_unit(smallest_unit %||% "nanosecond")
    largest_unit <- arg_unit(largest_unit %||% "auto", auto = TRUE)
    args <- with_relative(x, relative_to)
    return(new_duration_fields(zietig_call(rs_duration_round(
      duration_data(args$x), largest_unit, smallest_unit, rounding_increment, rounding_mode,
      args$relative
    ))))
  }
  if (!is.null(largest_unit) || !is.null(relative_to)) {
    zietig_type_error("`largest_unit` and `relative_to` are only used for durations.")
  }
  if (is.null(smallest_unit)) {
    zietig_type_error("`smallest_unit` must be given.")
  }
  smallest_unit <- arg_unit(smallest_unit)
  if (is_plain_time(x)) {
    f <- vec_data(x)
    new_plain_time_fields(zietig_call(rs_plain_time_round(
      f$second_of_day, f$nanos, smallest_unit, rounding_increment, rounding_mode
    )))
  } else if (is_plain_date_time(x)) {
    new_plain_date_time_fields(zietig_call(rs_plain_date_time_round(
      unclass(vec_data(x)), smallest_unit, rounding_increment, rounding_mode
    )))
  } else if (is_instant(x)) {
    new_instant_fields(zietig_call(rs_instant_round(
      instant_data(x), smallest_unit, rounding_increment, rounding_mode
    )))
  } else if (is_zoned_date_time(x)) {
    new_zoned_fields(zietig_call(rs_zoned_round(
      zoned_data(x), smallest_unit, rounding_increment, rounding_mode
    )))
  } else {
    zietig_type_error(sprintf("Can't round %s.", obj_type_friendly(x)))
  }
}

# Operators ---------------------------------------------------------------

is_temporal_point <- function(x) {
  is_plain_date(x) || is_plain_time(x) || is_plain_date_time(x) || is_instant(x) ||
    is_zoned_date_time(x)
}

temporal_arith <- function(op, x, y) {
  if (is_duration(y) && (op == "+" || op == "-")) {
    if (op == "-") y <- -y
    return(temporal_add_impl(x, y, reject = FALSE))
  }
  if (op == "-" && identical(class(x), class(y))) {
    return(temporal_since(x, y))
  }
  stop_incompatible_op(op, x, y)
}

#' @export
#' @method vec_arith zietig_plain_date
vec_arith.zietig_plain_date <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_plain_date", y)
}

#' @export
#' @method vec_arith.zietig_plain_date default
vec_arith.zietig_plain_date.default <- function(op, x, y, ...) temporal_arith(op, x, y)

#' @export
#' @method vec_arith zietig_plain_time
vec_arith.zietig_plain_time <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_plain_time", y)
}

#' @export
#' @method vec_arith.zietig_plain_time default
vec_arith.zietig_plain_time.default <- function(op, x, y, ...) temporal_arith(op, x, y)

#' @export
#' @method vec_arith zietig_plain_date_time
vec_arith.zietig_plain_date_time <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_plain_date_time", y)
}

#' @export
#' @method vec_arith.zietig_plain_date_time default
vec_arith.zietig_plain_date_time.default <- function(op, x, y, ...) temporal_arith(op, x, y)

#' @export
#' @method vec_arith zietig_instant
vec_arith.zietig_instant <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_instant", y)
}

#' @export
#' @method vec_arith.zietig_instant default
vec_arith.zietig_instant.default <- function(op, x, y, ...) temporal_arith(op, x, y)

#' @export
#' @method vec_arith zietig_zoned_date_time
vec_arith.zietig_zoned_date_time <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_zoned_date_time", y)
}

#' @export
#' @method vec_arith.zietig_zoned_date_time default
vec_arith.zietig_zoned_date_time.default <- function(op, x, y, ...) temporal_arith(op, x, y)

#' @export
#' @method vec_arith zietig_duration
vec_arith.zietig_duration <- function(op, x, y, ...) {
  UseMethod("vec_arith.zietig_duration", y)
}

#' @export
#' @method vec_arith.zietig_duration default
vec_arith.zietig_duration.default <- function(op, x, y, ...) {
  if (op == "+" && is_temporal_point(y)) {
    return(temporal_add_impl(y, x, reject = FALSE))
  }
  stop_incompatible_op(op, x, y)
}

#' @export
#' @method vec_arith.zietig_duration zietig_duration
vec_arith.zietig_duration.zietig_duration <- function(op, x, y, ...) {
  if (op != "+" && op != "-") {
    stop_incompatible_op(op, x, y)
  }
  if (op == "-") y <- -y
  args <- vec_recycle_common(x = x, y = y)
  new_duration_fields(zietig_call(
    rs_duration_add(duration_data(args$x), duration_data(args$y))
  ))
}

#' @export
#' @method vec_arith.zietig_duration MISSING
vec_arith.zietig_duration.MISSING <- function(op, x, y, ...) {
  switch(op,
    "-" = duration_map(x, function(v) -v),
    "+" = x,
    stop_incompatible_op(op, x, y)
  )
}
