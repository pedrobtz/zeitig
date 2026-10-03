#' Plain times
#'
#' A `zudate_plain_time` is a wall-clock time of day with nanosecond precision
#' and no date or time zone, the equivalent of
#' [`Temporal.PlainTime`](https://tc39.es/proposal-temporal/docs/plaintime.html).
#'
#' `plain_time()` builds times from their components (recycled to a common
#' length; all default to zero) or, when given a single character vector,
#' parses ISO 8601 / RFC 9557 strings such as `"15:23:30.123"` or
#' `"2019-11-18T15:23:30"` (the date part is ignored).
#'
#' @param hour,minute,second,millisecond,microsecond,nanosecond
#'   Integer-valued numbers. Fractions are truncated towards zero. `hour` may
#'   instead be a character vector to parse.
#' @param ... These dots are for future extensions and must be empty.
#' @param overflow How to handle out-of-range values: `"constrain"` (the
#'   default) clamps every component to its range (e.g. `hour = 25` becomes 23),
#'   `"reject"` raises an error.
#' @returns A `zudate_plain_time` vector.
#' @family plain time
#' @export
#' @examples
#' plain_time(19, 39, 9, 68, 346, 205)
#' plain_time(c("03:24:30", "15:23:30.5", NA))
#' plain_time(hour = 25, minute = 61) # constrained to 23:59:00
plain_time <- function(hour = 0L, minute = 0L, second = 0L, millisecond = 0L,
                       microsecond = 0L, nanosecond = 0L, ...,
                       overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  only_hour <- missing(minute) && missing(second) && missing(millisecond) &&
    missing(microsecond) && missing(nanosecond)
  if (is.character(hour) && only_hour) {
    return(plain_time_parse(hour))
  }
  reject <- arg_overflow(overflow)
  f <- recycle_fields(list(
    hour = as_int_field(hour),
    minute = as_int_field(minute),
    second = as_int_field(second),
    millisecond = as_int_field(millisecond),
    microsecond = as_int_field(microsecond),
    nanosecond = as_int_field(nanosecond)
  ))
  new_plain_time_fields(plain_time_parts(f, reject))
}

plain_time_parts <- function(f, reject, call = rlang::caller_env()) {
  zudate_call(
    rs_plain_time_from_parts(
      f$hour, f$minute, f$second, f$millisecond, f$microsecond, f$nanosecond, reject
    ),
    call = call
  )
}

plain_time_parse <- function(x, call = rlang::caller_env()) {
  check_character(x, call = call)
  new_plain_time_fields(zudate_call(rs_plain_time_parse(unname(x)), call = call))
}

new_plain_time_fields <- function(fields) {
  new_rcrd(fields, class = "zudate_plain_time")
}

#' @rdname plain_time
#' @param x An object to test.
#' @export
is_plain_time <- function(x) {
  inherits(x, "zudate_plain_time")
}

#' @export
as.character.zudate_plain_time <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zudate_plain_time <- function(x, ...) "ptime"

#' @export
vec_ptype_full.zudate_plain_time <- function(x, ...) "plain_time"

#' @export
vec_ptype2.zudate_plain_time.zudate_plain_time <- function(x, y, ...) {
  new_plain_time_fields(list(second_of_day = integer(), nanos = integer()))
}

#' @export
vec_cast.zudate_plain_time.zudate_plain_time <- function(x, to, ...) x

#' @export
vec_cast.zudate_plain_time.character <- function(x, to, ...) plain_time_parse(x)

#' @export
vec_cast.character.zudate_plain_time <- function(x, to, ...) format(x)
