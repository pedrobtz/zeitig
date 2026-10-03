#' Available time zones
#'
#' Lists the IANA time zone identifiers known to the time zone database used
#' by zeitig. On Linux and macOS this is the system database (`/usr/share/zoneinfo`
#' or the directory in the `TZDIR` environment variable); on Windows it is the
#' copy of the IANA database bundled with the package.
#'
#' This is the equivalent of `Intl.supportedValuesOf("timeZone")` in
#' JavaScript, which is what Temporal uses to validate time zone identifiers.
#'
#' @returns A sorted character vector of time zone identifiers.
#' @export
#' @examples
#' head(available_time_zones())
#' "Europe/Lisbon" %in% available_time_zones()
available_time_zones <- function() {
  rs_available_time_zones()
}
