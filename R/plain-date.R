#' Plain dates
#'
#' A `zeitig_plain_date` is a calendar date without a time or time zone, the
#' equivalent of
#' [`Temporal.PlainDate`](https://tc39.es/proposal-temporal/docs/plaindate.html).
#' Only the ISO 8601 calendar is supported.
#'
#' `plain_date()` builds dates from their components (recycled to a common
#' length) or, when given a single character vector, parses
#' [RFC 9557](https://www.rfc-editor.org/rfc/rfc9557) / ISO 8601 strings.
#' Strings that also contain a time, an offset or a time zone annotation are
#' accepted and those parts are ignored, but a UTC designator (`Z`) is an
#' error, as in Temporal.
#'
#' @param year,month,day Integer-valued numbers. Fractions are truncated
#'   towards zero. `year` may instead be a character vector to parse.
#' @param ... These dots are for future extensions and must be empty.
#' @param overflow How to handle out-of-range values: `"constrain"` (the
#'   default) clamps `month` to 12 and `day` to the length of the month,
#'   `"reject"` raises an error. Zero or negative months and days are always an
#'   error.
#' @returns A `zeitig_plain_date` vector.
#' @family plain date
#' @export
#' @examples
#' plain_date(2006, 8, 24)
#' plain_date(2021, 2, 31) # constrained to 2021-02-28
#' try(plain_date(2021, 2, 31, overflow = "reject"))
#' plain_date(c("2006-08-24", "2019-11-18T15:23:30+01:00[Europe/Paris]", NA))
plain_date <- function(year, month, day, ..., overflow = c("constrain", "reject")) {
  rlang::check_dots_empty0(...)
  if (is.character(year) && missing(month) && missing(day)) {
    return(plain_date_parse(year))
  }
  reject <- arg_overflow(overflow)
  f <- recycle_fields(list(
    year = as_int_field(year),
    month = as_int_field(month),
    day = as_int_field(day)
  ))
  fields <- zeitig_call(rs_plain_date_from_parts(f$year, f$month, f$day, reject))
  new_plain_date_fields(fields)
}

plain_date_parse <- function(x, call = rlang::caller_env()) {
  check_character(x, call = call)
  new_plain_date_fields(zeitig_call(rs_plain_date_parse(unname(x)), call = call))
}

new_plain_date_fields <- function(fields) {
  new_rcrd(fields, class = "zeitig_plain_date")
}

#' @rdname plain_date
#' @param x An object to test or convert.
#' @export
is_plain_date <- function(x) {
  inherits(x, "zeitig_plain_date")
}

#' @export
as.character.zeitig_plain_date <- function(x, ...) {
  format(x)
}

#' @export
vec_ptype_abbr.zeitig_plain_date <- function(x, ...) "pdate"

#' @export
vec_ptype_full.zeitig_plain_date <- function(x, ...) "plain_date"

#' @export
vec_ptype2.zeitig_plain_date.zeitig_plain_date <- function(x, y, ...) {
  new_plain_date_fields(list(year = integer(), month = integer(), day = integer()))
}

#' @export
vec_cast.zeitig_plain_date.zeitig_plain_date <- function(x, to, ...) x

#' @export
vec_cast.zeitig_plain_date.character <- function(x, to, ...) plain_date_parse(x)

#' @export
vec_cast.character.zeitig_plain_date <- function(x, to, ...) format(x)

#' @export
vec_cast.zeitig_plain_date.Date <- function(x, to, ...) as_plain_date(x)

#' @export
vec_cast.Date.zeitig_plain_date <- function(x, to, ...) as.Date(x)
