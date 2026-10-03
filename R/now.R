#' The current time
#'
#' The equivalent of `Temporal.Now`: the current exact time as an instant,
#' or as a zoned date-time or plain value in a time zone.
#'
#' The default time zone is the session's: the `TZ` environment variable
#' when it names a valid zone, otherwise [Sys.timezone()], otherwise `"UTC"`.
#' `now_time_zone()` returns that identifier. The same default applies to
#' `POSIXct` values without a `tzone` attribute.
#'
#' @param time_zone A time zone identifier, or `NULL` for the session time
#'   zone.
#' @returns A length-one vector of the requested type (`now_time_zone()`: a
#'   string).
#' @export
#' @examples
#' now_instant()
#' now_zoned_date_time("Asia/Tokyo")
#' now_plain_date()
#' now_plain_time("UTC")
#' now_plain_date_time()
#' now_time_zone()
now_instant <- function() {
  new_instant_fields(zudate_call(rs_now()))
}

#' @rdname now_instant
#' @export
now_zoned_date_time <- function(time_zone = NULL) {
  time_zone <- time_zone %||% now_time_zone()
  instant_to_zoned(instant_data(now_instant()), time_zone)
}

#' @rdname now_instant
#' @export
now_plain_date_time <- function(time_zone = NULL) {
  to_plain_date_time(now_zoned_date_time(time_zone))
}

#' @rdname now_instant
#' @export
now_plain_date <- function(time_zone = NULL) {
  to_plain_date(now_zoned_date_time(time_zone))
}

#' @rdname now_instant
#' @export
now_plain_time <- function(time_zone = NULL) {
  to_plain_time(now_zoned_date_time(time_zone))
}

#' @rdname now_instant
#' @export
now_time_zone <- function() {
  default_time_zone()
}

default_time_zone <- function() {
  # TZ first: Sys.timezone() caches its answer and misses later changes.
  for (tz in c(Sys.getenv("TZ"), Sys.timezone())) {
    if (length(tz) == 1L && !is.na(tz) && nzchar(tz)) {
      ok <- tryCatch(zudate_call(rs_time_zone_canonical(tz)), error = function(e) NA_character_)
      if (!is.na(ok)) {
        return(ok)
      }
    }
  }
  "UTC"
}
