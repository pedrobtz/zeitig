# Runs R code in a fresh R session with zeitig loaded and returns its output
# (stdout and stderr). The time zone database is read once per session, so
# `ZEITIG_TZDIR` can only be tested in a new process. The child uses the
# installed package under R CMD check and covr, and the source tree (through
# pkgload, without recompiling) under devtools::test().
zeitig_subprocess <- function(code, env = character()) {
  ns_path <- getNamespaceInfo(asNamespace("zeitig"), "path")
  if (dir.exists(file.path(ns_path, "src"))) {
    skip_if_not(nzchar(system.file(package = "pkgload")), "pkgload is not installed")
    load <- sprintf("pkgload::load_all(%s, compile = FALSE, quiet = TRUE)", deparse(ns_path))
  } else {
    load <- sprintf("library(zeitig, lib.loc = %s)", deparse(dirname(ns_path)))
  }
  set_env <- sprintf("Sys.setenv(%s = %s)", names(env), vapply(env, deparse, character(1)))
  script <- tempfile(fileext = ".R")
  on.exit(unlink(script))
  lib_paths <- paste(deparse(.libPaths()), collapse = "")
  writeLines(c(sprintf(".libPaths(%s)", lib_paths), set_env, load, code), script)
  rscript <- file.path(R.home("bin"), "Rscript")
  suppressWarnings(system2(rscript, c("--vanilla", shQuote(script)), stdout = TRUE, stderr = TRUE))
}

# A TZif (RFC 8536) file for a zone with a single fixed offset.
tzif_fixed <- function(seconds, abbr) {
  be32 <- function(x) as.raw(bitwAnd(bitwShiftR(x, c(24L, 16L, 8L, 0L)), 255L))
  counts <- c(be32(0L), be32(0L), be32(0L), be32(0L), be32(1L), be32(nchar(abbr) + 1L))
  header <- c(charToRaw("TZif2"), raw(15), counts)
  data <- c(be32(seconds), as.raw(0), as.raw(0), charToRaw(abbr), as.raw(0))
  footer <- charToRaw(sprintf("\n<%s>%d\n", abbr, -seconds %/% 3600L))
  c(header, data, header, data, footer)
}

# A fresh directory under the session's temporary directory (removed with it
# when the session ends).
new_test_dir <- function() {
  dir <- tempfile("zeitig-")
  dir.create(dir)
  dir
}
