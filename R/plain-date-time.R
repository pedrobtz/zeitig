#' Plain date-times
#'
#' A `zudate_plain_date_time` is a calendar date and a wall-clock time with
#' nanosecond precision but no time zone, the equivalent of
#' [`Temporal.PlainDateTime`](https://tc39.es/proposal-temporal/docs/plaindatetime.html).
#'
#' `plain_date_time()` builds date-times from their components (recycled to
#' a common length; the time components default to zero) or, when given a
#' single character vector, parses RFC 9557 / ISO 8601 strings. A string with
#' only a date is midnight of that day; offsets and time zone annotations are
#' ignored, but a UTC designator (`Z`) is an error, as in Temporal.
#'
#' @inheritParams plain_date
#' @inheritParams plain_time
#' @param year,month,day Integer-valued numbers. `year` may instead be a
#'   character vector to parse.
#' @param overflow How to handle out-of-range values: `"constrain"` (the
#'   default) clamps each component to its range, `"reject"` raises an error.
#' @returns A `zudate_plain_date_time` vector.
#' @family plain date-time
#' @export
#' @examples
#' plain_date_time(1995, 12, 7, 3, 24, 30, 0, 3, 500)
#' plain_date_time(c("1995-12-07T03:24:30.0000035", "2020-01-01"))
plain_date_time <- function(year, month, day, hour = 0L, minute = 0L, second = 0L,
                            millisecond = 0L, microsecond = 0L, nanosecond = 0L, ...,
                            overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  if (is.character(year) && missing(month) && missing(day)) {
    return(plain_date_time_parse(year))
  }
  reject <- arg_overflow(overflow)
  f <- recycle_fields(list(
    year = as_int_field(year),
    month = as_int_field(month),
    day = as_int_field(day),
    hour = as_int_field(hour),
    minute = as_int_field(minute),
    second = as_int_field(second),
    millisecond = as_int_field(millisecond),
    microsecond = as_int_field(microsecond),
    nanosecond = as_int_field(nanosecond)
  ))
  date <- zudate_call(rs_plain_date_from_parts(f$year, f$month, f$day, reject))
  time <- plain_time_parts(f, reject)
  new_plain_date_time_from(date, time)
}

plain_date_time_parse <- function(x, call = rlang::caller_env()) {
  check_character(x, call = call)
  new_plain_date_time_fields(zudate_call(rs_plain_date_time_parse(unname(x)), call = call))
}

new_plain_date_time_fields <- function(fields) {
  new_rcrd(fields, class = "zudate_plain_date_time")
}

# Combine date fields and time fields (lists or records) of the same length;
# a missing date or time makes the whole element missing.
new_plain_date_time_from <- function(date, time) {
  if (is_record(date)) date <- vec_data(date)
  if (is_record(time)) time <- vec_data(time)
  na <- is.na(date$year) | is.na(time$second_of_day)
  fields <- c(date[c("year", "month", "day")], time[c("second_of_day", "nanos")])
  fields <- lapply(fields, function(v) {
    v[na] <- NA_integer_
    v
  })
  new_plain_date_time_fields(fields)
}

is_record <- function(x) inherits(x, "vctrs_rcrd")

#' @rdname plain_date_time
#' @param x An object to test.
#' @export
is_plain_date_time <- function(x) {
  inherits(x, "zudate_plain_date_time")
}

#' @export
format.zudate_plain_date_time <- function(x, ...) {
  f <- vec_data(x)
  zudate_call(rs_plain_date_time_format(f$year, f$month, f$day, f$second_of_day, f$nanos))
}

#' @export
as.character.zudate_plain_date_time <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zudate_plain_date_time <- function(x, ...) "pdttm"

#' @export
vec_ptype_full.zudate_plain_date_time <- function(x, ...) "plain_date_time"

#' @export
vec_ptype2.zudate_plain_date_time.zudate_plain_date_time <- function(x, y, ...) {
  new_plain_date_time_fields(list(
    year = integer(), month = integer(), day = integer(),
    second_of_day = integer(), nanos = integer()
  ))
}

#' @export
vec_cast.zudate_plain_date_time.zudate_plain_date_time <- function(x, to, ...) x

#' @export
vec_cast.zudate_plain_date_time.character <- function(x, to, ...) plain_date_time_parse(x)

#' @export
vec_cast.character.zudate_plain_date_time <- function(x, to, ...) format(x)
