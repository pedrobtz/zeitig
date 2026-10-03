#' Zoned date-times
#'
#' A `zeitig_zoned_date_time` is an exact time together with the time zone
#' used to view it, the equivalent of
#' [`Temporal.ZonedDateTime`](https://tc39.es/proposal-temporal/docs/zoneddatetime.html)
#' (and of `jiff::Zoned`). Each element carries its own time zone: an IANA
#' identifier (see [available_time_zones()]) or a fixed offset such as
#' `"+05:30"`. Arithmetic with calendar units follows the wall clock and is
#' DST-aware.
#'
#' `zoned_date_time()` builds values from wall-clock components and a time
#' zone, resolving local times that fall in a DST gap or overlap with
#' `disambiguation`. Given a single character vector it parses RFC 9557
#' strings, which must carry a time zone annotation such as
#' `"2020-03-08T03:30-04:00[America/New_York]"`; `offset` says what to do when
#' the string's UTC offset disagrees with the time zone.
#'
#' @inheritParams plain_date_time
#' @param year,month,day Integer-valued numbers, or for `year` a character
#'   vector to parse.
#' @param time_zone Time zone identifiers, recycled against the components.
#' @param disambiguation For local times that occur twice (overlaps) or not at
#'   all (gaps): `"compatible"` (default; the earlier time in an overlap, the
#'   later in a gap), `"earlier"`, `"later"` or `"reject"`.
#' @param offset When parsing: `"reject"` (default) errors if the offset in
#'   the string is not valid for the time zone, `"use"` keeps the exact time
#'   given by the offset, `"ignore"` keeps the wall-clock time, `"prefer"` uses
#'   the offset when it is valid and the wall-clock time otherwise.
#' @returns A `zeitig_zoned_date_time` vector.
#' @family zoned date-time
#' @export
#' @examples
#' zoned_date_time(1995, 12, 7, 3, 24, 30, time_zone = "America/New_York")
#' zoned_date_time("1995-12-07T03:24:30-08:00[America/Los_Angeles]")
#' # 02:30 does not exist on 2019-03-10 in New York
#' zoned_date_time(2019, 3, 10, 2, 30, time_zone = "America/New_York")
#' zoned_date_time(2019, 3, 10, 2, 30, time_zone = "America/New_York", disambiguation = "earlier")
#' zoned_date_time(2020, 1, 1, time_zone = c("UTC", "Asia/Tokyo", "+05:30"))
zoned_date_time <- function(year, month, day, hour = 0L, minute = 0L, second = 0L,
                            millisecond = 0L, microsecond = 0L, nanosecond = 0L, ...,
                            time_zone,
                            disambiguation = c("compatible", "earlier", "later", "reject"),
                            offset = c("reject", "use", "prefer", "ignore"),
                            overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  disambiguation <- arg_match(disambiguation)
  if (is.character(year) && missing(month) && missing(day)) {
    offset <- arg_match(offset)
    return(zoned_date_time_parse(year, disambiguation, offset))
  }
  if (missing(time_zone)) {
    zeitig_type_error("`time_zone` must be given.")
  }
  pdt <- plain_date_time(
    year, month, day, hour, minute, second, millisecond, microsecond, nanosecond,
    overflow = overflow
  )
  zoned_from_plain(pdt, time_zone, disambiguation)
}

zoned_date_time_parse <- function(x, disambiguation = "compatible", offset = "reject",
                                  call = rlang::caller_env()) {
  check_character(x, call = call)
  new_zoned_fields(zeitig_call(rs_zoned_parse(unname(x), disambiguation, offset), call = call))
}

zoned_from_plain <- function(pdt, time_zone, disambiguation = "compatible", offset = NULL,
                             offset_mode = "prefer", call = rlang::caller_env()) {
  check_time_zone(time_zone, call = call)
  offset <- offset %||% NA_integer_
  args <- vec_recycle_common(x = pdt, tz = time_zone, offset = offset, .call = call)
  new_zoned_fields(zeitig_call(
    rs_zoned_from_civil(
      unclass(vec_data(args$x)), args$tz, disambiguation, args$offset, offset_mode
    ),
    call = call
  ))
}

check_time_zone <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (!is.character(x)) {
    zeitig_type_error(
      sprintf("`%s` must be a character vector of time zone identifiers.", arg),
      call = call
    )
  }
}

new_zoned_fields <- function(fields) {
  new_rcrd(fields, class = "zeitig_zoned_date_time")
}

zoned_data <- function(x) {
  unclass(vec_data(x))
}

#' @rdname zoned_date_time
#' @param x An object to test.
#' @export
is_zoned_date_time <- function(x) {
  inherits(x, "zeitig_zoned_date_time")
}

#' @export
as.character.zeitig_zoned_date_time <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zeitig_zoned_date_time <- function(x, ...) "zdttm"

#' @export
vec_ptype_full.zeitig_zoned_date_time <- function(x, ...) "zoned_date_time"

#' @export
vec_ptype2.zeitig_zoned_date_time.zeitig_zoned_date_time <- function(x, y, ...) {
  new_zoned_fields(list(seconds = double(), nanos = integer(), tz = character()))
}

#' @export
vec_cast.zeitig_zoned_date_time.zeitig_zoned_date_time <- function(x, to, ...) x

#' @export
vec_cast.zeitig_zoned_date_time.character <- function(x, to, ...) zoned_date_time_parse(x)

#' @export
vec_cast.character.zeitig_zoned_date_time <- function(x, to, ...) format(x)

# `==`, `<`, sort() and unique() use the exact time only, like
# ZonedDateTime.compare(); temporal_equals() also compares the time zone.
#' @export
vec_proxy_equal.zeitig_zoned_date_time <- function(x, ...) {
  new_data_frame(zoned_data(x)[c("seconds", "nanos")])
}

#' @export
vec_proxy_compare.zeitig_zoned_date_time <- function(x, ...) {
  new_data_frame(zoned_data(x)[c("seconds", "nanos")])
}

#' Time zone information
#'
#' * `time_zone()`: the time zone identifier of each element.
#' * `offset()`, `offset_nanoseconds()`: the UTC offset in effect, as a
#'   `"+HH:MM"` string or a number of nanoseconds.
#' * `hours_in_day()`: the length of the calendar day in hours (23 or 25 on
#'   DST transition days in many zones).
#' * `start_of_day()`: the first instant of the day (not always midnight).
#' * `time_zone_transition()`: the next or previous UTC offset change, `NA`
#'   when there is none.
#' * `with_time_zone()`: the same exact time viewed in another time zone.
#'
#' @param x A zoned date-time.
#' @returns See above.
#' @family zoned date-time
#' @export
#' @examples
#' x <- zoned_date_time("2020-03-08T12:00-04:00[America/New_York]")
#' time_zone(x)
#' offset(x)
#' hours_in_day(x)
#' start_of_day(x)
#' time_zone_transition(x, "previous")
#' time_zone_transition(x, "next")
#' with_time_zone(x, "Asia/Tokyo")
time_zone <- function(x) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  zoned_data(x)$tz
}

#' @rdname time_zone
#' @export
offset <- function(x) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  zeitig_call(rs_zoned_offset(zoned_data(x)))$string
}

#' @rdname time_zone
#' @export
offset_nanoseconds <- function(x) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  zeitig_call(rs_zoned_offset(zoned_data(x)))$seconds * 1e9
}

#' @rdname time_zone
#' @export
hours_in_day <- function(x) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  zeitig_call(rs_zoned_hours_in_day(zoned_data(x)))
}

#' @rdname time_zone
#' @export
start_of_day <- function(x) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  new_zoned_fields(zeitig_call(rs_zoned_start_of_day(zoned_data(x))))
}

#' @rdname time_zone
#' @param direction `"next"` or `"previous"`.
#' @export
time_zone_transition <- function(x, direction = c("next", "previous")) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  direction <- arg_match(direction)
  new_zoned_fields(zeitig_call(rs_zoned_transition(zoned_data(x), direction == "next")))
}

#' @rdname time_zone
#' @param time_zone Time zone identifiers, recycled against `x`.
#' @export
with_time_zone <- function(x, time_zone) {
  check_class(x, "zeitig_zoned_date_time", "a zoned date-time")
  instant_to_zoned(instant_data(x), time_zone)
}

instant_to_zoned <- function(fields, time_zone, call = rlang::caller_env()) {
  check_time_zone(time_zone, call = call)
  inst <- new_instant_fields(fields)
  args <- vec_recycle_common(x = inst, tz = time_zone, .call = call)
  new_zoned_fields(zeitig_call(
    rs_instant_to_zoned(instant_data(args$x), args$tz),
    call = call
  ))
}
