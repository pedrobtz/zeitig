#' Format Temporal objects as strings
#'
#' `format()` and `as.character()` give Temporal's `toString()` output (RFC
#' 9557 / ISO 8601). The arguments mirror `toString()`'s options; the defaults
#' print the shortest exact representation.
#'
#' * `fractional_second_digits`: `"auto"` (default; as many digits as needed)
#'   or a number from 0 to 9. The value is rounded with `rounding_mode` first.
#' * `smallest_unit`: `"minute"`, `"second"`, `"millisecond"`,
#'   `"microsecond"` or `"nanosecond"`; overrides `fractional_second_digits`.
#'   `"minute"` omits the seconds.
#' * `rounding_mode`: how to round to the requested precision (default
#'   `"trunc"`).
#' * `offset` (zoned): `"auto"` prints the UTC offset, `"never"` omits it.
#' * `time_zone_name` (zoned): `"auto"` prints the `[Zone]` annotation,
#'   `"never"` omits it and `"critical"` prints `[!Zone]`.
#' * `calendar_name` (dates and date-times): `"auto"`/`"never"` omit the
#'   calendar (always ISO 8601), `"always"` prints `[u-ca=iso8601]` and
#'   `"critical"` `[!u-ca=iso8601]`.
#' * `time_zone` (instants): print the wall-clock time and offset in this
#'   zone instead of UTC.
#'
#' @param x A Temporal object.
#' @param ... Not used; for compatibility with the generic.
#' @param fractional_second_digits,smallest_unit,rounding_mode See Details.
#' @param offset,time_zone_name,calendar_name,time_zone See Details. Each only
#'   applies to the types that have the corresponding part.
#' @returns A character vector, `NA` for missing elements.
#' @name temporal-format
#' @examples
#' x <- zoned_date_time("2020-01-01T15:23:30.123456789+01:00[Europe/Paris]")
#' format(x)
#' format(x, smallest_unit = "minute")
#' format(x, fractional_second_digits = 2, rounding_mode = "halfExpand")
#' format(x, offset = "never", time_zone_name = "critical", calendar_name = "always")
#' format(instant("2020-01-01T00:00Z"), time_zone = "Asia/Tokyo")
#' format(plain_time("12:00:00.5"), fractional_second_digits = 3)
NULL

format_options <- function(x, kind, fractional_second_digits = "auto", smallest_unit = NULL,
                           rounding_mode = "trunc", offset = "auto", time_zone_name = "auto",
                           calendar_name = "auto", time_zone = NULL,
                           call = rlang::caller_env()) {
  rounding_mode <- arg_rounding_mode(rounding_mode, call = call)
  offset <- arg_match(offset, c("auto", "never"), error_call = call)
  time_zone_name <- arg_match(time_zone_name, c("auto", "never", "critical"), error_call = call)
  calendar_name <- arg_match(
    calendar_name, c("auto", "always", "never", "critical"),
    error_call = call
  )
  digits <- -1L
  minute <- FALSE
  round <- NULL
  if (!is.null(smallest_unit)) {
    smallest_unit <- sub("s$", "", smallest_unit)
    smallest_unit <- arg_match(
      smallest_unit, c("minute", "second", "millisecond", "microsecond", "nanosecond"),
      error_call = call
    )
    digits <- switch(smallest_unit,
      minute = -1L,
      second = 0L,
      millisecond = 3L,
      microsecond = 6L,
      nanosecond = 9L
    )
    minute <- smallest_unit == "minute"
    round <- list(unit = smallest_unit, increment = 1)
  } else if (!identical(fractional_second_digits, "auto")) {
    fsd <- fractional_second_digits
    valid <- is.numeric(fsd) && length(fsd) == 1 && !is.na(fsd) && fsd %in% 0:9
    if (!valid) {
      zietig_range_error(
        "`fractional_second_digits` must be \"auto\" or a whole number from 0 to 9.",
        call = call
      )
    }
    digits <- as.integer(fractional_second_digits)
    round <- if (digits == 0L) {
      list(unit = "second", increment = 1)
    } else if (digits <= 3L) {
      list(unit = "millisecond", increment = 10^(3L - digits))
    } else if (digits <= 6L) {
      list(unit = "microsecond", increment = 10^(6L - digits))
    } else {
      list(unit = "nanosecond", increment = 10^(9L - digits))
    }
  }
  if (!is.null(round) && kind != "plain_date") {
    x <- temporal_round(
      x, round$unit,
      rounding_increment = round$increment, rounding_mode = rounding_mode
    )
  }
  if (is.null(time_zone)) {
    time_zone <- NA_character_
  } else {
    check_time_zone(time_zone, call = call)
    time_zone <- vec_recycle(time_zone, vec_size(x), call = call)
  }
  fields <- unclass(vec_data(x))
  zietig_call(
    rs_format(
      fields, kind, digits, minute, offset, time_zone_name, calendar_name, time_zone
    ),
    call = call
  )
}

#' @rdname temporal-format
#' @export
format.zietig_plain_date <- function(x, ..., calendar_name = "auto") {
  if (identical(calendar_name, "auto")) {
    f <- vec_data(x)
    return(zietig_call(rs_plain_date_format(f$year, f$month, f$day)))
  }
  format_options(x, "plain_date", calendar_name = calendar_name)
}

#' @rdname temporal-format
#' @export
format.zietig_plain_time <- function(x, ..., fractional_second_digits = "auto",
                                     smallest_unit = NULL, rounding_mode = "trunc") {
  if (is_default_precision(fractional_second_digits, smallest_unit)) {
    f <- vec_data(x)
    return(zietig_call(rs_plain_time_format(f$second_of_day, f$nanos)))
  }
  format_options(x, "plain_time", fractional_second_digits, smallest_unit, rounding_mode)
}

#' @rdname temporal-format
#' @export
format.zietig_plain_date_time <- function(x, ..., fractional_second_digits = "auto",
                                          smallest_unit = NULL, rounding_mode = "trunc",
                                          calendar_name = "auto") {
  defaults <- is_default_precision(fractional_second_digits, smallest_unit)
  if (defaults && identical(calendar_name, "auto")) {
    f <- vec_data(x)
    return(zietig_call(rs_plain_date_time_format(
      f$year, f$month, f$day, f$second_of_day, f$nanos
    )))
  }
  format_options(
    x, "plain_date_time", fractional_second_digits, smallest_unit, rounding_mode,
    calendar_name = calendar_name
  )
}

#' @rdname temporal-format
#' @export
format.zietig_instant <- function(x, ..., fractional_second_digits = "auto",
                                  smallest_unit = NULL, rounding_mode = "trunc",
                                  time_zone = NULL) {
  defaults <- is_default_precision(fractional_second_digits, smallest_unit)
  if (defaults && is.null(time_zone)) {
    return(zietig_call(rs_instant_format(instant_data(x))))
  }
  format_options(
    x, "instant", fractional_second_digits, smallest_unit, rounding_mode,
    time_zone = time_zone
  )
}

#' @rdname temporal-format
#' @export
format.zietig_zoned_date_time <- function(x, ..., fractional_second_digits = "auto",
                                          smallest_unit = NULL, rounding_mode = "trunc",
                                          offset = "auto", time_zone_name = "auto",
                                          calendar_name = "auto") {
  defaults <- is_default_precision(fractional_second_digits, smallest_unit) &&
    identical(offset, "auto") && identical(time_zone_name, "auto") &&
    identical(calendar_name, "auto")
  if (defaults) {
    return(zietig_call(rs_zoned_format(zoned_data(x))))
  }
  format_options(
    x, "zoned_date_time", fractional_second_digits, smallest_unit, rounding_mode,
    offset, time_zone_name, calendar_name
  )
}

is_default_precision <- function(fractional_second_digits, smallest_unit) {
  identical(fractional_second_digits, "auto") && is.null(smallest_unit)
}

temporal_kind <- function(x, call = rlang::caller_env()) {
  if (is_plain_date(x)) {
    "plain_date"
  } else if (is_plain_time(x)) {
    "plain_time"
  } else if (is_plain_date_time(x)) {
    "plain_date_time"
  } else if (is_instant(x)) {
    "instant"
  } else if (is_zoned_date_time(x)) {
    "zoned_date_time"
  } else {
    zietig_type_error(
      sprintf("`x` must be a Temporal date or time, not %s.", obj_type_friendly(x)),
      call = call
    )
  }
}

#' strftime-style formatting and parsing
#'
#' `temporal_strftime()` formats Temporal objects and `temporal_strptime()`
#' parses strings with `strftime`-style format strings, as implemented by
#' [`jiff::fmt::strtime`](https://docs.rs/jiff/latest/jiff/fmt/strtime/).
#' Common directives: `%Y` year, `%m` month, `%d` day, `%H` hour, `%M`
#' minute, `%S` second, `%f` fractional seconds, `%A`/`%a` weekday name,
#' `%B`/`%b` month name, `%j` day of year, `%z` UTC offset, `%Z` time zone
#' abbreviation, `%Q` IANA time zone name.
#'
#' Formatting fails for directives the value cannot supply (e.g. `%H` for a
#' plain date, `%z` for a plain date-time). Parsing to an instant needs an
#' offset (`%z`), and to a zoned date-time an offset or time zone name
#' (`%z`, `%Q`).
#'
#' @param x A Temporal object (`temporal_strftime()`) or a character vector
#'   (`temporal_strptime()`).
#' @param format Format strings, recycled against `x`.
#' @param class The class to parse into.
#' @returns A character vector (`temporal_strftime()`) or a vector of the
#'   requested class (`temporal_strptime()`).
#' @export
#' @examples
#' temporal_strftime(plain_date(2024, 7, 15), "%A, %B %d, %Y")
#' z <- zoned_date_time("2024-07-15T16:24:59-04:00[America/New_York]")
#' temporal_strftime(z, "%H:%M %Z (%z)")
#' temporal_strptime("15/07/2024", "%d/%m/%Y", "plain_date")
#' temporal_strptime("2024-07-15 16:24 -0400", "%Y-%m-%d %H:%M %z", "instant")
temporal_strftime <- function(x, format) {
  kind <- temporal_kind(x)
  check_character(format)
  args <- vec_recycle_common(x = x, format = format)
  zietig_call(rs_strftime(unclass(vec_data(args$x)), kind, args$format))
}

#' @rdname temporal_strftime
#' @export
temporal_strptime <- function(x, format,
                              class = c(
                                "plain_date", "plain_time", "plain_date_time", "instant",
                                "zoned_date_time"
                              )) {
  check_character(x)
  check_character(format)
  class <- arg_match(class)
  args <- vec_recycle_common(x = unname(x), format = format)
  fields <- zietig_call(rs_strptime(args$x, args$format, class))
  switch(class,
    plain_date = new_plain_date_fields(fields),
    plain_time = new_plain_time_fields(fields),
    plain_date_time = new_plain_date_time_fields(fields),
    instant = new_instant_fields(fields),
    zoned_date_time = new_zoned_fields(fields)
  )
}
