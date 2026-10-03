duration_field_names <- c(
  "years", "months", "weeks", "days", "hours", "minutes", "seconds",
  "milliseconds", "microseconds", "nanoseconds"
)

#' Durations
#'
#' A `zeitig_duration` is a length of time expressed in calendar and clock
#' units, the equivalent of
#' [`Temporal.Duration`](https://tc39.es/proposal-temporal/docs/duration.html)
#' (and of `jiff::Span`). Each element stores ten integer fields that must all
#' have the same sign; they are not balanced, so `duration(minutes = 90)`
#' stays 90 minutes.
#'
#' `duration()` builds durations from fields (recycled to a common length,
#' all defaulting to zero) or, when given a single character vector, parses
#' ISO 8601 duration strings such as `"P1Y2M3DT4H5M6.5S"`. Use
#' [temporal_fields()] (or `vctrs::field()`) to read the fields.
#'
#' Durations support unary `-`, `abs()`, `sign()`, `+` and `-` between
#' durations (calendar units need [temporal_add()] on a date instead), and
#' arithmetic with dates and times (see [temporal_add()]). Comparison operators
#' order durations by their length with 24-hour days; durations with years,
#' months or weeks can only be compared with [duration_compare()] and a
#' `relative_to` date. `==` compares the fields, so `PT1H == PT60M` is `FALSE`
#' (use `duration_compare()` to compare lengths).
#'
#' @param years,months,weeks,days,hours,minutes,seconds,milliseconds,microseconds,nanoseconds
#'   Integer-valued numbers (stored as doubles, so values beyond 2^53 lose
#'   precision). `years` may instead be a character vector to parse.
#' @param ... These dots are for future extensions and must be empty.
#' @returns A `zeitig_duration` vector.
#' @family duration
#' @export
#' @examples
#' duration(hours = 1, minutes = 30)
#' duration(c("P1Y2M3DT4H", "-PT1.5S", "PT0S", NA))
#' -duration(days = 3)
#' abs(duration("-P1D"))
#' duration(hours = 1) + duration(minutes = 90)
#' sort(duration(c("PT2H", "PT90M", "P1D")))
duration <- function(years = 0, months = 0, weeks = 0, days = 0, hours = 0, minutes = 0,
                     seconds = 0, milliseconds = 0, microseconds = 0, nanoseconds = 0, ...) {
  rlang::check_dots_empty0(...)
  if (is.character(years) && nargs() == 1L) {
    return(duration_parse(years))
  }
  fields <- list(
    years = years, months = months, weeks = weeks, days = days, hours = hours,
    minutes = minutes, seconds = seconds, milliseconds = milliseconds,
    microseconds = microseconds, nanoseconds = nanoseconds
  )
  fields <- Map(as_duration_field, fields, names(fields))
  fields <- recycle_fields(fields)
  new_duration_fields(zeitig_call(rs_duration_validate(fields)))
}

# Temporal's ToIntegerIfIntegral: fractions are a RangeError.
as_duration_field <- function(x, arg, call = rlang::caller_env(2)) {
  if (is.logical(x) && all(is.na(x))) {
    return(rep(NA_real_, length(x)))
  }
  if (!is.numeric(x)) {
    zeitig_type_error(
      sprintf("`%s` must be a number, not %s.", arg, obj_type_friendly(x)),
      call = call
    )
  }
  x <- as.double(unclass(x))
  bad <- !is.na(x) & (!is.finite(x) | x != trunc(x))
  if (any(bad)) {
    zeitig_range_error(
      sprintf("`%s` must be a finite integer (element %d).", arg, which(bad)[[1]]),
      call = call
    )
  }
  x
}

duration_parse <- function(x, call = rlang::caller_env()) {
  check_character(x, call = call)
  new_duration_fields(zeitig_call(rs_duration_parse(unname(x)), call = call))
}

new_duration_fields <- function(fields) {
  new_rcrd(fields, class = "zeitig_duration")
}

duration_data <- function(x) {
  unclass(vec_data(x))
}

#' @rdname duration
#' @param x An object to test.
#' @export
is_duration <- function(x) {
  inherits(x, "zeitig_duration")
}

#' @export
format.zeitig_duration <- function(x, ...) {
  zeitig_call(rs_duration_format(duration_data(x)))
}

#' @export
as.character.zeitig_duration <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zeitig_duration <- function(x, ...) "dur"

#' @export
vec_ptype_full.zeitig_duration <- function(x, ...) "duration"

#' @export
vec_ptype2.zeitig_duration.zeitig_duration <- function(x, y, ...) {
  new_duration_fields(rlang::rep_named(duration_field_names, list(double())))
}

#' @export
vec_cast.zeitig_duration.zeitig_duration <- function(x, to, ...) x

#' @export
vec_cast.zeitig_duration.character <- function(x, to, ...) duration_parse(x)

#' @export
vec_cast.character.zeitig_duration <- function(x, to, ...) format(x)

#' @export
vec_proxy_compare.zeitig_duration <- function(x, ...) {
  key <- zeitig_call(rs_duration_sort_key(duration_data(x)))
  new_data_frame(key)
}

#' @export
vec_math.zeitig_duration <- function(.fn, .x, ...) {
  switch(.fn,
    abs = duration_map(.x, abs),
    sign = duration_sign(.x),
    zeitig_type_error(sprintf("`%s()` is not supported for durations.", .fn))
  )
}

duration_map <- function(x, fn) {
  f <- lapply(duration_data(x), function(v) fn(v) + 0)
  new_duration_fields(f)
}

duration_sign <- function(x) {
  f <- duration_data(x)
  s <- Reduce(function(acc, v) ifelse(acc == 0, sign(v), acc), f, rep(0, vec_size(x)))
  as.integer(s)
}

#' Duration helpers
#'
#' * `duration_total()` expresses a duration as a (fractional) number of
#'   `unit`s (`Duration.prototype.total()`).
#' * `duration_compare()` compares two durations by length, returning -1, 0
#'   or 1 (`Duration.compare()`).
#' * `duration_blank()` is `TRUE` for zero durations (`.blank`).
#'
#' Without `relative_to`, days are 24 hours long and durations with years,
#' months are an error. With `relative_to` (a plain date, plain date-time or
#' zoned date-time, or a string), calendar units are resolved from that
#' starting point; with a zoned date-time, days follow the time zone's DST
#' rules.
#'
#' @param x,y Durations (or ISO 8601 strings), recycled to a common length.
#' @param unit The unit to express the duration in, e.g. `"hour"`. Plural
#'   spellings (`"hours"`) are accepted.
#' @param relative_to `NULL`, or a plain date, plain date-time or zoned
#'   date-time (or string) recycled against `x`.
#' @returns `duration_total()`: a double vector. `duration_compare()`: an
#'   integer vector. `duration_blank()`: a logical vector.
#' @family duration
#' @export
#' @examples
#' duration_total(duration(hours = 1, minutes = 30), "minute")
#' duration_total(duration(months = 1), "day", relative_to = plain_date(2020, 2, 1))
#' duration_compare(duration(hours = 1), duration(minutes = 60))
#' duration_compare(duration(months = 1), duration(days = 30), relative_to = "2020-02-01")
#' duration_blank(duration(c("PT0S", "PT1S")))
duration_total <- function(x, unit, relative_to = NULL) {
  x <- as_duration(x)
  unit <- arg_unit(unit)
  args <- with_relative(x, relative_to)
  zeitig_call(rs_duration_total(duration_data(args$x), unit, args$relative))
}

#' @rdname duration_total
#' @export
duration_compare <- function(x, y, relative_to = NULL) {
  args <- vec_recycle_common(x = as_duration(x), y = as_duration(y))
  rel <- with_relative(args$x, relative_to)
  y <- vec_recycle(args$y, vec_size(rel$x))
  zeitig_call(rs_duration_compare(duration_data(rel$x), duration_data(y), rel$relative))
}

#' @rdname duration_total
#' @export
duration_blank <- function(x) {
  x <- as_duration(x)
  s <- duration_sign(x)
  s == 0L
}

# Temporal parses a relativeTo string as a ZonedDateTime when it carries a
# time zone annotation and as a PlainDate(Time) otherwise.
parse_relative_to <- function(x, call = rlang::caller_env()) {
  if (all(is.na(x) | grepl("[", x, fixed = TRUE))) {
    zoned_date_time_parse(x, call = call)
  } else {
    plain_date_time_parse(x, call = call)
  }
}

# Recycle a duration with its `relative_to` and turn the latter into the list
# of plain date-time fields Rust expects (or NULL).
with_relative <- function(x, relative_to, call = rlang::caller_env()) {
  if (is.null(relative_to)) {
    return(list(x = x, relative = NULL))
  }
  if (is.character(relative_to)) {
    relative_to <- parse_relative_to(relative_to, call = call)
  }
  if (is_zoned_date_time(relative_to)) {
    args <- vec_recycle_common(x = x, relative = relative_to, .call = call)
    return(list(x = args$x, relative = zoned_data(args$relative)))
  }
  if (is_plain_date(relative_to)) {
    relative_to <- to_plain_date_time(relative_to)
  } else if (!is_plain_date_time(relative_to)) {
    zeitig_type_error(
      sprintf(
        "`relative_to` must be a plain date, plain date-time or zoned date-time, not %s.",
        obj_type_friendly(relative_to)
      ),
      call = call
    )
  }
  args <- vec_recycle_common(x = x, relative = relative_to, .call = call)
  list(x = args$x, relative = unclass(vec_data(args$relative)))
}

#' Coerce to and from durations
#'
#' `as_duration()` converts ISO 8601 strings and `difftime` objects to
#' durations; `as_difftime()` converts durations without calendar units
#' (years, months, weeks) to `difftime`, treating days as 24 hours.
#'
#' A `difftime` in days, hours, minutes or seconds keeps that unit when its
#' values are whole numbers; otherwise (and for weeks) it becomes seconds plus
#' a sub-second part rounded to the nanosecond.
#'
#' @param x An object to convert.
#' @param ... Passed on to methods.
#' @param units The units of the result, as in [base::difftime()].
#' @returns A duration (`as_duration()`) or a `difftime` (`as_difftime()`).
#' @family duration
#' @export
#' @examples
#' as_duration(as.difftime(90, units = "mins"))
#' as_duration(as.difftime(1.5, units = "hours"))
#' as_difftime(duration(hours = 1, minutes = 30))
#' as_difftime(duration(hours = 36), units = "days")
as_duration <- function(x, ...) UseMethod("as_duration")

#' @export
as_duration.default <- function(x, ...) {
  zeitig_type_error(sprintf("Can't convert %s to a duration.", obj_type_friendly(x)))
}

#' @export
as_duration.zeitig_duration <- function(x, ...) x

#' @export
as_duration.character <- function(x, ...) duration_parse(x)

#' @export
as_duration.difftime <- function(x, ...) {
  units <- attr(x, "units")
  v <- as.double(unclass(x))
  field <- c(secs = "seconds", mins = "minutes", hours = "hours", days = "days")[units]
  if (!is.na(field) && all(is.na(v) | (is.finite(v) & v == trunc(v)))) {
    args <- list(v)
    names(args) <- field
    return(do.call(duration, args))
  }
  secs <- as.double(x, units = "secs")
  whole <- trunc(secs)
  ns <- round((secs - whole) * 1e9)
  a <- abs(ns)
  s <- sign(ns)
  duration(
    seconds = whole,
    milliseconds = trunc(a / 1e6) * s,
    microseconds = trunc(a / 1e3) %% 1e3 * s,
    nanoseconds = a %% 1e3 * s
  )
}

#' @rdname as_duration
#' @export
as_difftime <- function(x, ..., units = "auto") UseMethod("as_difftime")

#' @export
as_difftime.zeitig_duration <- function(x, ..., units = "auto") {
  secs <- duration_total(x, "second")
  out <- as.difftime(secs, units = "secs")
  if (!identical(units, "secs")) {
    if (identical(units, "auto")) {
      units <- auto_difftime_units(secs)
    }
    units(out) <- units
  }
  out
}

auto_difftime_units <- function(secs) {
  m <- suppressWarnings(min(abs(secs), na.rm = TRUE))
  if (!is.finite(m) || m < 60) {
    "secs"
  } else if (m < 3600) {
    "mins"
  } else if (m < 86400) {
    "hours"
  } else {
    "days"
  }
}

# Units ------------------------------------------------------------------

temporal_units <- c(
  "year", "month", "week", "day", "hour", "minute", "second",
  "millisecond", "microsecond", "nanosecond"
)

# Validate a unit name, accepting Temporal's plural spellings.
arg_unit <- function(x, auto = FALSE, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  values <- c(temporal_units, paste0(temporal_units, "s"))
  if (auto) {
    values <- c("auto", values)
  }
  x <- arg_match(x, values, error_arg = arg, error_call = call)
  sub("s$", "", x)
}

rounding_modes <- c(
  "ceil", "floor", "expand", "trunc", "halfCeil", "halfFloor", "halfExpand", "halfTrunc",
  "halfEven"
)

arg_rounding_mode <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  arg_match(x, rounding_modes, error_arg = arg, error_call = call)
}

arg_increment <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (!is.numeric(x) || length(x) != 1 || is.na(x)) {
    zeitig_type_error(sprintf("`%s` must be a single number.", arg), call = call)
  }
  as.double(x)
}
