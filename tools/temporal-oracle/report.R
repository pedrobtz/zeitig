# Summarises the Temporal conformance test: for each type, how many fixture
# cases match Temporal, how many are documented differences, and which.
# The figures are quoted in vignettes/temporal-differences.Rmd.
#
# Usage (from the package root): Rscript tools/temporal-oracle/report.R

suppressMessages(devtools::load_all(quiet = TRUE))
env <- new.env(parent = asNamespace("zeitig"))
sys.source("tests/testthat/helper-conformance.R", envir = env)
res <- do.call(rbind, lapply(env$conformance_types, function(type) {
  cbind(type = type, env$conformance_run(type))
}))
tab <- table(factor(res$type, env$conformance_types), res$status)
print(addmargins(tab, 1))
cat(sprintf(
  "\n%d cases: %.1f%% match Temporal, %.1f%% are documented differences, %d mismatches\n",
  nrow(res), 100 * mean(res$status == "match"),
  100 * mean(res$status %in% c("documented", "skipped")), sum(res$status == "mismatch")
))
cat("\nDocumented differences by key (design.md section 9):\n")
print(table(res$divergence))
