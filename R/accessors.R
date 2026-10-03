# Field extraction shared by the accessors. Each helper returns the civil
# fields of `x` as a list of integer vectors, or errors for types that have no
# such fields.

civil_date_fields <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (is_plain_date(x) || is_plain_date_time(x)) {
    return(vec_data(x)[c("year", "month", "day")])
  }
  if (is_zoned_date_time(x)) {
    return(zoned_civil(x, call = call)[c("year", "month", "day")])
  }
  zietig_type_error(
    sprintf(
      "`%s` must be a plain date, plain date-time or zoned date-time, not %s.",
      arg, obj_type_friendly(x)
    ),
    call = call
  )
}

civil_time_fields <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (is_plain_time(x) || is_plain_date_time(x)) {
    return(vec_data(x)[c("second_of_day", "nanos")])
  }
  if (is_zoned_date_time(x)) {
    return(zoned_civil(x, call = call)[c("second_of_day", "nanos")])
  }
  zietig_type_error(
    sprintf(
      "`%s` must be a plain time, plain date-time or zoned date-time, not %s.",
      arg, obj_type_friendly(x)
    ),
    call = call
  )
}

# Wall-clock fields of a zoned date-time, as a list of plain date-time fields.
zoned_civil <- function(x, call = rlang::caller_env()) {
  zietig_call(rs_zoned_civil(zoned_data(x)), call = call)
}

date_field <- function(x, field, call = rlang::caller_env()) {
  f <- civil_date_fields(x, arg = "x", call = call)
  zietig_call(rs_plain_date_field(f$year, f$month, f$day, field), call = call)
}

time_field <- function(x, field, call = rlang::caller_env()) {
  f <- civil_time_fields(x, arg = "x", call = call)
  sod <- f$second_of_day
  ns <- f$nanos
  switch(field,
    hour = sod %/% 3600L,
    minute = sod %/% 60L %% 60L,
    second = sod %% 60L,
    millisecond = ns %/% 1000000L,
    microsecond = ns %/% 1000L %% 1000L,
    nanosecond = ns %% 1000L
  )
}

#' Date and time fields
#'
#' Accessors for the fields of Temporal objects, the equivalent of the
#' getters such as `.year`, `.dayOfWeek` or `.inLeapYear`. All are vectorised
#' and return `NA` for missing elements.
#'
#' * `year()`, `month()`, `day()`: the calendar date (ISO 8601 calendar).
#' * `hour()`, `minute()`, `second()`, `millisecond()`, `microsecond()`,
#'   `nanosecond()`: the wall-clock time; `second()` is the whole second and
#'   the sub-second part is split into the three remaining fields, each in
#'   `0:999`.
#' * `day_of_week()`: 1 (Monday) to 7 (Sunday).
#' * `day_of_year()`: 1 to 366.
#' * `week_of_year()`, `year_of_week()`: the ISO 8601 week number and the
#'   year that week belongs to (which differs from `year()` around New Year).
#' * `days_in_week()`, `days_in_month()`, `days_in_year()`,
#'   `months_in_year()`, `in_leap_year()`.
#'
#' `temporal_fields()` returns all the component fields as a data frame; for
#' durations these are the ten fields from `years` to `nanoseconds`.
#'
#' @param x A Temporal object: date fields need a plain date, plain
#'   date-time or zoned date-time (whose fields are the wall clock in its time
#'   zone), time fields a plain time, plain date-time or zoned date-time;
#'   `temporal_fields()` also accepts durations.
#' @returns An integer vector (`in_leap_year()`: logical; `temporal_fields()`:
#'   a data frame).
#' @name temporal-fields
#' @examples
#' x <- plain_date_time("2021-01-03T15:23:30.123456789")
#' year(x)
#' day_of_week(x)
#' week_of_year(x)
#' year_of_week(x)
#' c(millisecond(x), microsecond(x), nanosecond(x))
#' temporal_fields(x)
NULL

#' @rdname temporal-fields
#' @export
year <- function(x) civil_date_fields(x)$year

#' @rdname temporal-fields
#' @export
month <- function(x) civil_date_fields(x)$month

#' @rdname temporal-fields
#' @export
day <- function(x) civil_date_fields(x)$day

#' @rdname temporal-fields
#' @export
hour <- function(x) time_field(x, "hour")

#' @rdname temporal-fields
#' @export
minute <- function(x) time_field(x, "minute")

#' @rdname temporal-fields
#' @export
second <- function(x) time_field(x, "second")

#' @rdname temporal-fields
#' @export
millisecond <- function(x) time_field(x, "millisecond")

#' @rdname temporal-fields
#' @export
microsecond <- function(x) time_field(x, "microsecond")

#' @rdname temporal-fields
#' @export
nanosecond <- function(x) time_field(x, "nanosecond")

#' @rdname temporal-fields
#' @export
day_of_week <- function(x) date_field(x, "day_of_week")

#' @rdname temporal-fields
#' @export
day_of_year <- function(x) date_field(x, "day_of_year")

#' @rdname temporal-fields
#' @export
week_of_year <- function(x) date_field(x, "week_of_year")

#' @rdname temporal-fields
#' @export
year_of_week <- function(x) date_field(x, "year_of_week")

#' @rdname temporal-fields
#' @export
days_in_week <- function(x) {
  f <- civil_date_fields(x)
  ifelse(is.na(f$year), NA_integer_, 7L)
}

#' @rdname temporal-fields
#' @export
days_in_month <- function(x) date_field(x, "days_in_month")

#' @rdname temporal-fields
#' @export
days_in_year <- function(x) date_field(x, "days_in_year")

#' @rdname temporal-fields
#' @export
months_in_year <- function(x) {
  f <- civil_date_fields(x)
  ifelse(is.na(f$year), NA_integer_, 12L)
}

#' @rdname temporal-fields
#' @export
in_leap_year <- function(x) as.logical(date_field(x, "in_leap_year"))

#' @rdname temporal-fields
#' @export
temporal_fields <- function(x) {
  if (is_duration(x)) {
    return(new_data_frame(duration_data(x), n = vec_size(x)))
  }
  out <- list()
  if (is_plain_date(x) || is_plain_date_time(x) || is_zoned_date_time(x)) {
    out <- c(out, civil_date_fields(x))
  }
  if (is_plain_time(x) || is_plain_date_time(x) || is_zoned_date_time(x)) {
    out <- c(out, lapply(
      c(
        hour = "hour", minute = "minute", second = "second", millisecond = "millisecond",
        microsecond = "microsecond", nanosecond = "nanosecond"
      ),
      function(field) time_field(x, field)
    ))
  }
  if (length(out) == 0) {
    zietig_type_error(sprintf("`x` must be a Temporal object, not %s.", obj_type_friendly(x)))
  }
  new_data_frame(out, n = vec_size(x))
}
