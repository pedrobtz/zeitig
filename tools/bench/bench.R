# Throughput of common zeitig operations, with base R equivalents where one
# exists. The numbers quoted in design.md section 10 come from this script.
#
# Benchmark an installed (release) build, not devtools::load_all(), which
# compiles the much slower cargo dev profile:
#
#   R CMD INSTALL .
#   Rscript tools/bench/bench.R            # n = 1e6 by default
#   Rscript tools/bench/bench.R 1e5        # smaller n
#
# Each timing is the median of `reps` runs; the last column is nanoseconds
# per element.

suppressPackageStartupMessages(library(zeitig))
args <- commandArgs(trailingOnly = TRUE)
n <- if (length(args)) as.integer(as.numeric(args[[1]])) else 1e6L
reps <- 5L
set.seed(1)

time_it <- function(expr) {
  expr <- substitute(expr)
  env <- parent.frame()
  times <- vapply(seq_len(reps), function(i) {
    gc(FALSE)
    system.time(eval(expr, env))[["elapsed"]]
  }, numeric(1))
  stats::median(times)
}

secs <- as.double(1.5e9 + sample.int(1e8, n))
ct <- as.POSIXct(secs, tz = "Europe/Paris")
dates <- as.Date(ct)
inst <- instant_from_epoch(seconds = secs)
zoned <- to_zoned_date_time(inst, "Europe/Paris")
zoned4 <- to_zoned_date_time(
  inst, sample(c("Europe/Paris", "America/New_York", "Asia/Tokyo", "UTC"), n, TRUE)
)
pdt <- to_plain_date_time(zoned)
pd <- to_plain_date(pdt)
pt <- to_plain_time(pdt)
hours <- duration(hours = rep(1, n))
one_day <- duration(days = 1)
zoned_str <- format(zoned)
inst_str <- format(inst)
pdt_str <- format(pdt)

cases <- list(
  "parse: instant()" = quote(instant(inst_str)),
  "parse: plain_date_time()" = quote(plain_date_time(pdt_str)),
  "parse: zoned_date_time()" = quote(zoned_date_time(zoned_str)),
  "build: plain_date() from fields" = quote(plain_date(2020, 1, 1 + integer(n))),
  "build: plain_date_time() from fields" =
    quote(plain_date_time(2020, 1, 1, 12, 0, 1 + integer(n))),
  "build: duration() from fields" = quote(duration(hours = rep(1, n))),
  "format: plain_date" = quote(format(pd)),
  "format: plain_date_time" = quote(format(pdt)),
  "format: instant" = quote(format(inst)),
  "format: zoned" = quote(format(zoned)),
  "format: zoned, fractional_second_digits = 3" =
    quote(format(zoned, fractional_second_digits = 3)),
  "add: plain_date + P1D" = quote(pd + one_day),
  "add: plain_date_time + PT1H" = quote(pdt + hours),
  "add: instant + PT1H" = quote(inst + hours),
  "add: zoned + P1D (one zone)" = quote(zoned + one_day),
  "add: zoned + P1D (four zones)" = quote(zoned4 + one_day),
  "add: duration + duration" = quote(hours + hours),
  "until: instant" = quote(temporal_until(inst, inst)),
  "until: zoned, largest_unit = day" = quote(temporal_until(zoned, zoned, largest_unit = "day")),
  "until: zoned, halfEven to days" = quote(temporal_until(
    zoned, zoned, largest_unit = "day", smallest_unit = "day", rounding_mode = "halfEven"
  )),
  "round: plain_time to minute" = quote(temporal_round(pt, "minute")),
  "round: zoned to hour" = quote(temporal_round(zoned, "hour")),
  "fields: year(zoned)" = quote(year(zoned)),
  "fields: offset(zoned)" = quote(offset(zoned)),
  "fields: temporal_fields(zoned)" = quote(temporal_fields(zoned)),
  "with: temporal_with(zoned, hour = 1)" = quote(temporal_with(zoned, hour = 1)),
  "sort: duration" = quote(sort(hours)),
  "convert: as.POSIXct(plain_date_time)" = quote(as.POSIXct(pdt)),
  "convert: as_plain_date_time(POSIXct)" = quote(as_plain_date_time(ct)),
  "base: Date + 1" = quote(dates + 1),
  "base: format(Date)" = quote(format(dates)),
  "base: format(POSIXct)" = quote(format(ct, "%Y-%m-%dT%H:%M:%S")),
  "base: as.POSIXlt(POSIXct)$year" = quote(as.POSIXlt(ct)$year)
)

res <- vapply(cases, function(e) time_it(eval(e)), numeric(1))
out <- data.frame(
  operation = names(cases),
  seconds = round(res, 3),
  ns_per_element = round(res / n * 1e9),
  row.names = NULL
)
cat(sprintf(
  "zeitig %s, n = %s, median of %d runs\n\n",
  packageVersion("zeitig"), format(n, big.mark = ","), reps
))
print(out, right = FALSE)
