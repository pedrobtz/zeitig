#' Instants
#'
#' A `zudate_instant` is an exact point in time with nanosecond precision and
#' no time zone or calendar, the equivalent of
#' [`Temporal.Instant`](https://tc39.es/proposal-temporal/docs/instant.html)
#' (and of `jiff::Timestamp`). The supported range is years -9999 to 9999.
#'
#' `instant()` parses RFC 9557 strings, which must include a UTC offset or
#' `Z` (a time zone annotation alone is not enough). `instant_from_epoch()`
#' builds instants from a count of seconds, milliseconds or nanoseconds since
#' 1970-01-01T00:00Z; nanoseconds may be given as strings because doubles
#' cannot hold present-day epoch nanoseconds exactly.
#'
#' @param x A character vector to parse, or an object to test.
#' @returns A `zudate_instant` vector.
#' @family instant
#' @export
#' @examples
#' instant(c("1969-07-20T20:17Z", "2020-01-01T12:00:00.5+01:00", NA))
#' instant_from_epoch(seconds = 0)
#' instant_from_epoch(milliseconds = 1553906700000)
#' instant_from_epoch(nanoseconds = "1553906700000000001")
instant <- function(x) {
  check_character(x)
  new_instant_fields(zudate_call(rs_instant_parse(unname(x))))
}

#' @rdname instant
#' @param seconds,milliseconds,nanoseconds Exactly one of these: numbers of
#'   units since the epoch, or for `nanoseconds` also a character vector of
#'   integers. Fractional seconds are rounded to the microsecond; fractional
#'   milliseconds and nanoseconds are truncated.
#' @export
instant_from_epoch <- function(seconds = NULL, milliseconds = NULL, nanoseconds = NULL) {
  given <- !vapply(list(seconds, milliseconds, nanoseconds), is.null, logical(1))
  if (sum(given) != 1L) {
    zudate_type_error("Exactly one of `seconds`, `milliseconds` and `nanoseconds` must be given.")
  }
  if (!is.null(nanoseconds)) {
    if (is.numeric(nanoseconds)) {
      nanoseconds <- ifelse(is.na(nanoseconds), NA_character_, format(
        trunc(nanoseconds),
        scientific = FALSE, trim = TRUE
      ))
    }
    check_character(nanoseconds)
    return(new_instant_fields(zudate_call(rs_instant_from_epoch_nanoseconds(nanoseconds))))
  }
  if (!is.null(seconds)) {
    s <- as_epoch_number(seconds)
    secs <- floor(s)
    # A double cannot hold present-day epoch seconds to the nanosecond, so
    # fractions are rounded to the microsecond (as for POSIXct).
    nanos <- round((s - secs) * 1e6) * 1e3
  } else {
    ms <- trunc(as_epoch_number(milliseconds))
    secs <- floor(ms / 1000)
    nanos <- (ms - secs * 1000) * 1e6
  }
  carry <- !is.na(nanos) & nanos >= 1e9
  secs[carry] <- secs[carry] + 1
  nanos[carry] <- 0
  new_instant_fields(zudate_call(rs_instant_validate(list(
    seconds = as.double(secs), nanos = as.integer(nanos)
  ))))
}

as_epoch_number <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (is.logical(x) && all(is.na(x))) {
    return(rep(NA_real_, length(x)))
  }
  if (!is.numeric(x)) {
    zudate_type_error(
      sprintf("`%s` must be a number, not %s.", arg, obj_type_friendly(x)),
      call = call
    )
  }
  x <- as.double(unclass(x))
  bad <- !is.na(x) & !is.finite(x)
  if (any(bad)) {
    zudate_range_error(sprintf("`%s` must be finite (element %d).", arg, which(bad)[[1]]),
      call = call
    )
  }
  x
}

new_instant_fields <- function(fields) {
  new_rcrd(fields, class = "zudate_instant")
}

instant_data <- function(x) {
  f <- unclass(vec_data(x))
  f[c("seconds", "nanos")]
}

#' @rdname instant
#' @export
is_instant <- function(x) {
  inherits(x, "zudate_instant")
}

#' Epoch time
#'
#' The time since 1970-01-01T00:00Z of an instant or zoned date-time.
#' `epoch_seconds()` and `epoch_milliseconds()` round towards negative
#' infinity, as Temporal's `epochMilliseconds`; `epoch_nanoseconds()` is exact
#' and returned as a character vector because R has no 64-bit integers.
#'
#' @param x An instant or zoned date-time.
#' @returns A double vector (`epoch_seconds()`, `epoch_milliseconds()`) or a
#'   character vector (`epoch_nanoseconds()`).
#' @family instant
#' @export
#' @examples
#' x <- instant("2019-03-30T00:45:00.123456789Z")
#' epoch_seconds(x)
#' epoch_milliseconds(x)
#' epoch_nanoseconds(x)
epoch_seconds <- function(x) {
  instant_fields_of(x)$seconds
}

#' @rdname epoch_seconds
#' @export
epoch_milliseconds <- function(x) {
  f <- instant_fields_of(x)
  f$seconds * 1000 + f$nanos %/% 1000000L
}

#' @rdname epoch_seconds
#' @export
epoch_nanoseconds <- function(x) {
  f <- instant_fields_of(x)
  zudate_call(rs_instant_epoch_nanoseconds(f))
}

instant_fields_of <- function(x, call = rlang::caller_env()) {
  if (is_instant(x) || is_zoned_date_time(x)) {
    return(instant_data(x))
  }
  zudate_type_error(
    sprintf("`x` must be an instant or zoned date-time, not %s.", obj_type_friendly(x)),
    call = call
  )
}

#' @export
format.zudate_instant <- function(x, ...) {
  zudate_call(rs_instant_format(instant_data(x)))
}

#' @export
as.character.zudate_instant <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zudate_instant <- function(x, ...) "inst"

#' @export
vec_ptype_full.zudate_instant <- function(x, ...) "instant"

#' @export
vec_ptype2.zudate_instant.zudate_instant <- function(x, y, ...) {
  new_instant_fields(list(seconds = double(), nanos = integer()))
}

#' @export
vec_cast.zudate_instant.zudate_instant <- function(x, to, ...) x

#' @export
vec_cast.zudate_instant.character <- function(x, to, ...) instant(x)

#' @export
vec_cast.character.zudate_instant <- function(x, to, ...) format(x)

#' @export
vec_cast.zudate_instant.POSIXct <- function(x, to, ...) as_instant(x)

#' @export
vec_cast.POSIXct.zudate_instant <- function(x, to, ...) {
  as.POSIXct(x, tz = attr(to, "tzone") %||% "")
}
