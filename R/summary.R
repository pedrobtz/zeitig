#' Summaries of Temporal vectors
#'
#' `summary()` gives the minimum, quartiles (as observed values, i.e.
#' quantile type 1), maximum and the number of missing values, in Temporal's
#' ordering. Durations are ordered by length with 24-hour days, so durations
#' with years or months cannot be summarized.
#'
#' @param object A Temporal vector.
#' @param ... Not used.
#' @returns A named character vector of class `table`.
#' @name temporal-summary
#' @examples
#' summary(plain_date(2020, 1:12, 1))
#' summary(duration(hours = c(1, 5, NA)))
NULL

temporal_summary <- function(object, ...) {
  missing <- vec_detect_missing(object)
  x <- vec_sort(vec_slice(object, !missing))
  n <- vec_size(x)
  labels <- c("Min.", "1st Qu.", "Median", "3rd Qu.", "Max.")
  if (n == 0) {
    values <- rep(NA_character_, 5)
  } else {
    pos <- pmax(1L, ceiling(c(0, 0.25, 0.5, 0.75, 1) * n))
    values <- format(vec_slice(x, pos))
  }
  out <- stats::setNames(values, labels)
  if (any(missing)) {
    out <- c(out, "NA's" = as.character(sum(missing)))
  }
  structure(out, class = "table")
}

#' @rdname temporal-summary
#' @export
summary.zietig_plain_date <- temporal_summary

#' @rdname temporal-summary
#' @export
summary.zietig_plain_time <- temporal_summary

#' @rdname temporal-summary
#' @export
summary.zietig_plain_date_time <- temporal_summary

#' @rdname temporal-summary
#' @export
summary.zietig_instant <- temporal_summary

#' @rdname temporal-summary
#' @export
summary.zietig_zoned_date_time <- temporal_summary

#' @rdname temporal-summary
#' @export
summary.zietig_duration <- temporal_summary
