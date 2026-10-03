#' Convert between Temporal types
#'
#' Conversions that drop or add information, mirroring Temporal's
#' `.toPlainDate()`, `.toPlainTime()`, `.toPlainDateTime()`, `.toInstant()`
#' and `.toZonedDateTime()`:
#'
#' * `to_plain_date()` and `to_plain_time()` take the date or time part of a
#'   plain date-time or (wall clock of a) zoned date-time.
#' * `to_plain_date_time()` combines a plain date with a plain time
#'   (midnight when `time` is `NULL`), or takes the wall clock of a zoned
#'   date-time.
#' * `to_instant()` drops the time zone of a zoned date-time.
#' * `to_zoned_date_time()` places an instant, plain date-time or plain date
#'   in `time_zone`. For plain types the local time is resolved with
#'   `disambiguation`; a plain date without `time` becomes the start of that
#'   day. A zoned date-time is converted to `time_zone` keeping the exact
#'   time.
#'
#' @param x A Temporal object.
#' @returns A plain date, plain time or plain date-time vector.
#' @name temporal-conversions
#' @examples
#' dt <- plain_date_time("1995-12-07T03:24:30")
#' to_plain_date(dt)
#' to_plain_time(dt)
#' to_plain_date_time(plain_date(2006, 8, 24), plain_time(15, 30))
#' z <- to_zoned_date_time(dt, "Europe/Paris")
#' z
#' to_instant(z)
#' to_zoned_date_time(instant("2020-01-01T00:00Z"), "Asia/Tokyo")
#' to_zoned_date_time(plain_date(2020, 3, 29), "Asia/Beirut") # day starts at 01:00
NULL

#' @rdname temporal-conversions
#' @export
to_plain_date <- function(x) {
  check_class(x, c("zietig_plain_date_time", "zietig_zoned_date_time"), "a date-time")
  new_plain_date_fields(civil_date_fields(x))
}

#' @rdname temporal-conversions
#' @export
to_plain_time <- function(x) {
  check_class(x, c("zietig_plain_date_time", "zietig_zoned_date_time"), "a date-time")
  new_plain_time_fields(civil_time_fields(x))
}

#' @rdname temporal-conversions
#' @param time A plain time (or string) recycled against `x`, or `NULL` for
#'   midnight.
#' @export
to_plain_date_time <- function(x, time = NULL) {
  if (is_zoned_date_time(x)) {
    return(new_plain_date_time_fields(zoned_civil(x)))
  }
  check_class(x, "zietig_plain_date", "a plain date or zoned date-time")
  if (is.null(time)) {
    time <- plain_time()
  }
  time <- as_plain_time(time)
  args <- vec_recycle_common(date = x, time = time)
  new_plain_date_time_from(args$date, args$time)
}

#' @rdname temporal-conversions
#' @export
to_instant <- function(x) {
  check_class(x, "zietig_zoned_date_time", "a zoned date-time")
  new_instant_fields(instant_data(x))
}

#' @rdname temporal-conversions
#' @param time_zone Time zone identifiers, recycled against `x`.
#' @param disambiguation How to resolve local times in DST gaps and overlaps,
#'   see [zoned_date_time()].
#' @export
to_zoned_date_time <- function(x, time_zone, time = NULL,
                               disambiguation = c("compatible", "earlier", "later", "reject")) {
  disambiguation <- arg_match(disambiguation)
  if (is_instant(x)) {
    return(instant_to_zoned(instant_data(x), time_zone))
  }
  if (is_zoned_date_time(x)) {
    return(with_time_zone(x, time_zone))
  }
  if (is_plain_date(x) && is.null(time)) {
    midnight <- zoned_from_plain(to_plain_date_time(x), time_zone, "compatible")
    return(start_of_day(midnight))
  }
  if (is_plain_date(x)) {
    x <- to_plain_date_time(x, time)
  }
  check_class(x, "zietig_plain_date_time", "an instant, plain date or plain date-time")
  zoned_from_plain(x, time_zone, disambiguation)
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
#' * `POSIXct` converts to an instant, or to a zoned date-time in its
#'   `tzone` (the session time zone, see [now_time_zone()], when unset);
#'   `as_zoned_date_time(x, time_zone = )` picks another zone.
#'
#' The reverse conversions use base generics: [as.Date()] for plain dates,
#' plain date-times and zoned date-times (wall-clock date), and
#' [as.POSIXct()] / [as.POSIXlt()]. For plain date-times these interpret the
#' wall-clock time in `tz` (UTC by default; local times in DST gaps are
#' resolved by the operating system). Instants and zoned date-times convert
#' exactly up to the microsecond; zoned date-times keep their time zone when
#' all elements share one IANA zone and `tz` is not given, otherwise `tz`
#' (default UTC) is used.
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
#' as_instant(as.POSIXct("2020-01-01 12:00:00", tz = "UTC"))
#' as_zoned_date_time(as.POSIXct("2020-07-01 12:00:00", tz = "Europe/Paris"))
#' as.POSIXct(zoned_date_time("2020-07-01T12:00+02:00[Europe/Paris]"))
NULL

#' @rdname temporal-coercion
#' @export
as_plain_date <- function(x, ...) UseMethod("as_plain_date")

#' @export
as_plain_date.default <- function(x, ...) {
  zietig_type_error(sprintf("Can't convert %s to a plain date.", obj_type_friendly(x)))
}

#' @export
as_plain_date.zietig_plain_date <- function(x, ...) x

#' @export
as_plain_date.zietig_plain_date_time <- function(x, ...) to_plain_date(x)

#' @export
as_plain_date.character <- function(x, ...) plain_date_parse(x)

#' @export
as_plain_date.Date <- function(x, ...) {
  new_plain_date_fields(zietig_call(rs_plain_date_from_epoch_days(as.double(unclass(x)))))
}

#' @export
as_plain_date.POSIXt <- function(x, ...) to_plain_date(as_plain_date_time(x))

#' @export
as_plain_date.zietig_zoned_date_time <- function(x, ...) to_plain_date(x)

#' @rdname temporal-coercion
#' @export
as_plain_time <- function(x, ...) UseMethod("as_plain_time")

#' @export
as_plain_time.default <- function(x, ...) {
  zietig_type_error(sprintf("Can't convert %s to a plain time.", obj_type_friendly(x)))
}

#' @export
as_plain_time.zietig_plain_time <- function(x, ...) x

#' @export
as_plain_time.zietig_plain_date_time <- function(x, ...) to_plain_time(x)

#' @export
as_plain_time.character <- function(x, ...) plain_time_parse(x)

#' @export
as_plain_time.POSIXt <- function(x, ...) to_plain_time(as_plain_date_time(x))

#' @export
as_plain_time.zietig_zoned_date_time <- function(x, ...) to_plain_time(x)

#' @rdname temporal-coercion
#' @export
as_plain_date_time <- function(x, ...) UseMethod("as_plain_date_time")

#' @export
as_plain_date_time.default <- function(x, ...) {
  zietig_type_error(sprintf("Can't convert %s to a plain date-time.", obj_type_friendly(x)))
}

#' @export
as_plain_date_time.zietig_plain_date_time <- function(x, ...) x

#' @export
as_plain_date_time.zietig_plain_date <- function(x, ...) to_plain_date_time(x)

#' @export
as_plain_date_time.character <- function(x, ...) plain_date_time_parse(x)

#' @export
as_plain_date_time.Date <- function(x, ...) to_plain_date_time(as_plain_date(x))

#' @export
as_plain_date_time.zietig_zoned_date_time <- function(x, ...) to_plain_date_time(x)

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
as.Date.zietig_plain_date <- function(x, ...) {
  f <- vec_data(x)
  days <- zietig_call(rs_plain_date_to_epoch_days(f$year, f$month, f$day))
  structure(days, class = "Date")
}

#' @export
as.Date.zietig_plain_date_time <- function(x, ...) {
  as.Date(to_plain_date(x))
}

#' @export
as.Date.zietig_zoned_date_time <- function(x, ...) {
  as.Date(to_plain_date(x))
}

#' @rdname temporal-coercion
#' @export
as_instant <- function(x, ...) UseMethod("as_instant")

#' @export
as_instant.default <- function(x, ...) {
  zietig_type_error(sprintf("Can't convert %s to an instant.", obj_type_friendly(x)))
}

#' @export
as_instant.zietig_instant <- function(x, ...) x

#' @export
as_instant.zietig_zoned_date_time <- function(x, ...) to_instant(x)

#' @export
as_instant.character <- function(x, ...) instant(x)

#' @export
as_instant.POSIXt <- function(x, ...) {
  instant_from_epoch(seconds = as.double(as.POSIXct(x)))
}

#' @rdname temporal-coercion
#' @export
as_zoned_date_time <- function(x, ...) UseMethod("as_zoned_date_time")

#' @export
as_zoned_date_time.default <- function(x, ...) {
  zietig_type_error(sprintf("Can't convert %s to a zoned date-time.", obj_type_friendly(x)))
}

#' @export
as_zoned_date_time.zietig_zoned_date_time <- function(x, ...) x

#' @export
as_zoned_date_time.character <- function(x, ...) zoned_date_time_parse(x)

#' @export
as_zoned_date_time.zietig_instant <- function(x, ..., time_zone) {
  if (missing(time_zone)) {
    zietig_type_error("`time_zone` must be given to convert an instant to a zoned date-time.")
  }
  to_zoned_date_time(x, time_zone)
}

#' @export
as_zoned_date_time.POSIXt <- function(x, ..., time_zone = NULL) {
  ct <- as.POSIXct(x)
  time_zone <- time_zone %||% posixct_time_zone(ct)
  to_zoned_date_time(as_instant(ct), time_zone)
}

posixct_time_zone <- function(x) {
  tz <- attr(x, "tzone")
  if (is.null(tz) || identical(tz[[1]], "")) default_time_zone() else tz[[1]]
}

#' @export
as.POSIXct.zietig_instant <- function(x, tz = "UTC", ...) {
  f <- instant_data(x)
  .POSIXct(f$seconds + f$nanos / 1e9, tz = tz)
}

#' @export
as.POSIXct.zietig_zoned_date_time <- function(x, tz = NULL, ...) {
  if (is.null(tz)) {
    zones <- unique(stats::na.omit(time_zone(x)))
    tz <- if (length(zones) == 1L && !grepl("^[+-]", zones)) zones else "UTC"
  }
  as.POSIXct(to_instant(x), tz = tz)
}

#' @export
as.POSIXlt.zietig_instant <- function(x, tz = "UTC", ...) {
  as.POSIXlt(as.POSIXct(x, tz = tz), tz = tz)
}

#' @export
as.POSIXlt.zietig_zoned_date_time <- function(x, tz = NULL, ...) {
  ct <- as.POSIXct(x, tz = tz)
  as.POSIXlt(ct, tz = attr(ct, "tzone"))
}

#' @export
as.POSIXct.zietig_plain_date_time <- function(x, tz = "UTC", ...) {
  f <- temporal_fields(x)
  sec <- f$second + (f$millisecond * 1e6 + f$microsecond * 1e3 + f$nanosecond) / 1e9
  ISOdatetime(f$year, f$month, f$day, f$hour, f$minute, sec, tz = tz)
}

#' @export
as.POSIXlt.zietig_plain_date_time <- function(x, tz = "UTC", ...) {
  as.POSIXlt(as.POSIXct(x, tz = tz), tz = tz)
}
