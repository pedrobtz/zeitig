#' Convert between Temporal types
#'
#' Conversions that drop or add information, mirroring Temporal's
#' `.toPlainDate()`, `.toPlainTime()` and `.toPlainDateTime()`:
#'
#' * `to_plain_date()` and `to_plain_time()` take the date or time part of a
#'   plain date-time.
#' * `to_plain_date_time()` combines a plain date with a plain time
#'   (midnight when `time` is `NULL`).
#'
#' @param x A Temporal object.
#' @returns A plain date, plain time or plain date-time vector.
#' @name temporal-conversions
#' @examples
#' dt <- plain_date_time("1995-12-07T03:24:30")
#' to_plain_date(dt)
#' to_plain_time(dt)
#' to_plain_date_time(plain_date(2006, 8, 24), plain_time(15, 30))
NULL

#' @rdname temporal-conversions
#' @export
to_plain_date <- function(x) {
  check_class(x, "zudate_plain_date_time", "a plain date-time")
  new_plain_date_fields(vec_data(x)[c("year", "month", "day")])
}

#' @rdname temporal-conversions
#' @export
to_plain_time <- function(x) {
  check_class(x, "zudate_plain_date_time", "a plain date-time")
  new_plain_time_fields(vec_data(x)[c("second_of_day", "nanos")])
}

#' @rdname temporal-conversions
#' @param time A plain time (or string) recycled against `x`, or `NULL` for
#'   midnight.
#' @export
to_plain_date_time <- function(x, time = NULL) {
  check_class(x, "zudate_plain_date", "a plain date")
  if (is.null(time)) {
    time <- plain_time()
  }
  time <- as_plain_time(time)
  args <- vec_recycle_common(date = x, time = time)
  new_plain_date_time_from(args$date, args$time)
}

#' Coerce to Temporal types
#'
#' S3 generics that convert other objects, including base R classes, to
#' Temporal types. Character vectors are parsed as RFC 9557 / ISO 8601
#' strings.
#'
#' * `Date` converts exactly to a plain date (and to midnight for a plain
#'   date-time).
#' * `POSIXct` and `POSIXlt` convert to the wall-clock date and time in their
#'   own time zone (the `tzone` attribute, or the session time zone). `POSIXct`
#'   values are rounded to the microsecond, because a double cannot hold
#'   nanoseconds of a present-day instant; `POSIXlt` seconds are rounded to the
#'   nanosecond.
#'
#' The reverse conversions use base generics: [as.Date()] for plain dates and
#' plain date-times, and [as.POSIXct()] / [as.POSIXlt()] for plain date-times,
#' which interpret the wall-clock time in `tz` (UTC by default). Local times
#' that do not exist in `tz` (DST gaps) are resolved by the operating system.
#'
#' @param x An object to convert.
#' @param ... Passed on to methods.
#' @returns A Temporal vector of the requested type.
#' @name temporal-coercion
#' @examples
#' as_plain_date(as.Date("2020-02-29"))
#' as_plain_date_time(as.POSIXct("2020-02-29 12:30:00", tz = "UTC"))
#' as_plain_time("12:30")
#' as.Date(plain_date(2020, 2, 29))
#' as.POSIXct(plain_date_time("2020-02-29T12:30"), tz = "America/New_York")
NULL

#' @rdname temporal-coercion
#' @export
as_plain_date <- function(x, ...) UseMethod("as_plain_date")

#' @export
as_plain_date.default <- function(x, ...) {
  zudate_type_error(sprintf("Can't convert %s to a plain date.", obj_type_friendly(x)))
}

#' @export
as_plain_date.zudate_plain_date <- function(x, ...) x

#' @export
as_plain_date.zudate_plain_date_time <- function(x, ...) to_plain_date(x)

#' @export
as_plain_date.character <- function(x, ...) plain_date_parse(x)

#' @export
as_plain_date.Date <- function(x, ...) {
  new_plain_date_fields(zudate_call(rs_plain_date_from_epoch_days(as.double(unclass(x)))))
}

#' @export
as_plain_date.POSIXt <- function(x, ...) to_plain_date(as_plain_date_time(x))

#' @rdname temporal-coercion
#' @export
as_plain_time <- function(x, ...) UseMethod("as_plain_time")

#' @export
as_plain_time.default <- function(x, ...) {
  zudate_type_error(sprintf("Can't convert %s to a plain time.", obj_type_friendly(x)))
}

#' @export
as_plain_time.zudate_plain_time <- function(x, ...) x

#' @export
as_plain_time.zudate_plain_date_time <- function(x, ...) to_plain_time(x)

#' @export
as_plain_time.character <- function(x, ...) plain_time_parse(x)

#' @export
as_plain_time.POSIXt <- function(x, ...) to_plain_time(as_plain_date_time(x))

#' @rdname temporal-coercion
#' @export
as_plain_date_time <- function(x, ...) UseMethod("as_plain_date_time")

#' @export
as_plain_date_time.default <- function(x, ...) {
  zudate_type_error(sprintf("Can't convert %s to a plain date-time.", obj_type_friendly(x)))
}

#' @export
as_plain_date_time.zudate_plain_date_time <- function(x, ...) x

#' @export
as_plain_date_time.zudate_plain_date <- function(x, ...) to_plain_date_time(x)

#' @export
as_plain_date_time.character <- function(x, ...) plain_date_time_parse(x)

#' @export
as_plain_date_time.Date <- function(x, ...) to_plain_date_time(as_plain_date(x))

#' @export
as_plain_date_time.POSIXct <- function(x, ...) {
  lt <- as.POSIXlt(x)
  posixlt_to_plain_date_time(lt, digits = 6L)
}

#' @export
as_plain_date_time.POSIXlt <- function(x, ...) {
  posixlt_to_plain_date_time(x, digits = 9L)
}

posixlt_to_plain_date_time <- function(lt, digits) {
  lt <- unclass(lt)
  sec <- lt$sec
  whole <- floor(sec)
  frac <- round((sec - whole) * 10^digits) * 10^(9L - digits)
  # Rounding up to a full second carries into the seconds field.
  carry <- !is.na(frac) & frac >= 1e9
  whole[carry] <- whole[carry] + 1
  frac[carry] <- 0
  frac <- as.integer(frac)
  plain_date_time(
    lt$year + 1900L, lt$mon + 1L, lt$mday, lt$hour, lt$min, whole,
    millisecond = frac %/% 1000000L,
    microsecond = frac %/% 1000L %% 1000L,
    nanosecond = frac %% 1000L
  )
}

#' @export
as.Date.zudate_plain_date <- function(x, ...) {
  f <- vec_data(x)
  days <- zudate_call(rs_plain_date_to_epoch_days(f$year, f$month, f$day))
  structure(days, class = "Date")
}

#' @export
as.Date.zudate_plain_date_time <- function(x, ...) {
  as.Date(to_plain_date(x))
}

#' @export
as.POSIXct.zudate_plain_date_time <- function(x, tz = "UTC", ...) {
  f <- temporal_fields(x)
  sec <- f$second + (f$millisecond * 1e6 + f$microsecond * 1e3 + f$nanosecond) / 1e9
  ISOdatetime(f$year, f$month, f$day, f$hour, f$minute, sec, tz = tz)
}

#' @export
as.POSIXlt.zudate_plain_date_time <- function(x, tz = "UTC", ...) {
  as.POSIXlt(as.POSIXct(x, tz = tz), tz = tz)
}
