# Compares zeitig with the Temporal reference implementation on the generated
# fixtures in fixtures/temporal/ (see helper-conformance.R and
# tools/temporal-oracle/). Every documented difference is recorded in the
# fixtures as a `divergence`; any other mismatch fails.

skip_if_not_installed("zujson")

for (type in conformance_types) {
  test_that(paste("zeitig matches Temporal:", type), {
    res <- conformance_run(type)
    expect(
      !any(res$status == "mismatch"),
      sprintf(
        "%d of %d cases differ from Temporal:\n%s",
        sum(res$status == "mismatch"), nrow(res), conformance_describe(res)
      )
    )
  })
}
