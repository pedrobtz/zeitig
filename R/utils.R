# Internal helpers shared by all types.

# Run a call into Rust and rethrow its errors as zeitig conditions. Rust only
# reports range problems (invalid values, unparsable strings); type problems
# are detected in R before the call.
zeitig_call <- function(expr, call = rlang::caller_env()) {
  tryCatch(expr, error = function(e) {
    abort(
      conditionMessage(e),
      class = c("zeitig_range_error", "zeitig_error"),
      call = call
    )
  })
}

zeitig_type_error <- function(message, call = rlang::caller_env()) {
  abort(message, class = c("zeitig_type_error", "zeitig_error"), call = call)
}

zeitig_range_error <- function(message, call = rlang::caller_env()) {
  abort(message, class = c("zeitig_range_error", "zeitig_error"), call = call)
}

# Temporal's ToIntegerWithTruncation: numbers are truncated towards zero,
# non-finite values are a RangeError and anything that is not a number is a
# TypeError. `NA` stays `NA`.
as_int_field <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (is.integer(x)) {
    return(unclass(x))
  }
  if (is.logical(x) && all(is.na(x))) {
    return(rep(NA_integer_, length(x)))
  }
  if (!is.double(x)) {
    zeitig_type_error(
      sprintf("`%s` must be a number, not %s.", arg, obj_type_friendly(x)),
      call = call
    )
  }
  x <- unclass(x)
  bad <- !is.na(x) & (!is.finite(x) | abs(x) > .Machine$integer.max)
  if (any(bad)) {
    zeitig_range_error(
      sprintf("`%s` must be finite and smaller than 2^31 (element %d).", arg, which(bad)[[1]]),
      call = call
    )
  }
  as.integer(trunc(x))
}

obj_type_friendly <- function(x) {
  if (is.null(x)) {
    return("NULL")
  }
  paste0("a <", class(x)[[1]], "> object")
}

arg_overflow <- function(overflow, call = rlang::caller_env()) {
  overflow <- arg_match(overflow, c("constrain", "reject"), error_call = call)
  identical(overflow, "reject")
}

check_character <- function(x, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (!is.character(x)) {
    zeitig_type_error(
      sprintf("`%s` must be a character vector, not %s.", arg, obj_type_friendly(x)),
      call = call
    )
  }
}

check_class <- function(x, class, what, arg = rlang::caller_arg(x), call = rlang::caller_env()) {
  if (!inherits(x, class)) {
    zeitig_type_error(
      sprintf("`%s` must be %s, not %s.", arg, what, obj_type_friendly(x)),
      call = call
    )
  }
}

# Recycle named arguments to a common size with vctrs' rules.
recycle_fields <- function(fields, call = rlang::caller_env()) {
  vec_recycle_common(!!!fields, .call = call)
}
