# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this package is

`zudate` is an R package that implements the [TC39 Temporal](https://tc39.es/proposal-temporal/docs/) date/time model
(Instant, PlainDate, PlainTime, PlainDateTime, ZonedDateTime, Duration, ...) on top of the Rust crate
[`jiff`](https://docs.rs/jiff), which is vendored into the package so it builds offline on CRAN.
The R <-> Rust bridge is [savvy](https://yutannihilation.github.io/savvy/guide/).

Read `design.md` (architecture, data model, API mapping, CRAN constraints) and `roadmap.md`
(milestones for 0.1.0) before adding features. Keep both files current when a design decision changes.

## Commands

All R commands are run from the package root. `devtools`, `testthat`, `roxygen2`, `rcmdcheck`, `lintr`,
`styler` and `pkgdown` are expected to be installed. `savvy-cli` is installed with
`cargo install savvy-cli --locked`.

```sh
# Regenerate the FFI glue after ANY change to #[savvy] items in src/rust/src/*.rs
savvy-cli update .            # rewrites src/init.c, src/rust/api.h, R/000-wrappers.R

# Regenerate NAMESPACE and man/ (roxygen comments live in R/*.R, including the generated wrappers)
Rscript -e 'devtools::document()'

# Compile the Rust crate + shared object and load the package into a session
Rscript -e 'devtools::load_all()'          # DEBUG=true -> cargo dev profile (pkgbuild sets it)

# R-side tests (testthat, edition 3)
Rscript -e 'devtools::test()'
Rscript -e 'devtools::test(filter = "instant")'     # single file: tests/testthat/test-instant.R
Rscript -e 'testthat::test_file("tests/testthat/test-instant.R")'

# Rust-only unit tests (no R session; #[cfg(test)] modules)
cargo test --manifest-path src/rust/Cargo.toml

# Rust tests that need a live R session (#[cfg(feature = "savvy-test")] modules)
savvy-cli test src/rust

# Full check as CRAN sees it (what CI runs)
Rscript -e 'rcmdcheck::rcmdcheck(args = c("--no-manual", "--as-cran"), error_on = "warning")'

# Rust lint / format
cargo fmt --manifest-path src/rust/Cargo.toml
cargo clippy --manifest-path src/rust/Cargo.toml --all-targets

# R lint / format
Rscript -e 'lintr::lint_package()'
Rscript -e 'styler::style_pkg()'

# Docs site
Rscript -e 'pkgdown::build_site()'
```

Order of operations when touching Rust: edit `src/rust/src/*.rs` -> `savvy-cli update .` ->
`devtools::document()` -> `devtools::load_all()` / `devtools::test()`. Forgetting `savvy-cli update`
leaves `init.c`/`api.h`/`000-wrappers.R` stale and produces "object 'savvy_..._impl' not found" or
link errors. Forgetting `document()` leaves `NAMESPACE` without `useDynLib`, which gives the same
"object not found" error even though compilation succeeded.

## Architecture

### Build pipeline (how Rust ends up inside the R package)

1. `configure` / `configure.win` check for `cargo`, print cargo/rustc versions (a CRAN requirement),
   pick the cargo profile (`release`, or `dev` when `DEBUG=true` / `SAVVY_PROFILE`), and `sed` the
   placeholders in `src/Makevars.in` / `src/Makevars.win.in` into `src/Makevars` / `src/Makevars.win`.
   The generated Makevars files are gitignored and removed by `cleanup` / `cleanup.win`.
2. `src/Makevars` runs `cargo build --lib` on `src/rust/Cargo.toml` producing the static library
   `libzudate.a`, then links it with `src/init.c` into `zudate.so` / `zudate.dll`. The static lib is
   deleted after linking (`clean_intermediate`) to keep the installed size small.
3. On Windows the Makevars additionally mocks `libgcc_eh.a` and points cargo at the Rtools linker;
   `src/zudate-win.def` exports only `R_init_zudate`.
4. `.Rbuildignore` excludes `src/rust/.cargo` and `src/rust/target` from the tarball.

For CRAN the crate dependencies (jiff, savvy, ...) must be vendored (`cargo vendor` ->
`src/rust/vendor.tar.xz`, extracted by Makevars and built with `--offline`). See `design.md`
"Vendoring and CRAN compliance" for the exact scheme and `roadmap.md` for its status.

### FFI layer (savvy)

- `src/rust/src/lib.rs` (and any modules it declares) is the only hand-written Rust. Functions and
  `impl` blocks marked `#[savvy]` are exported to R.
- `savvy-cli update .` parses those attributes and generates three files that must never be edited
  by hand: `src/rust/api.h` (C prototypes), `src/init.c` (`.Call` entry points, `handle_result`
  error trampoline, `R_init_zudate` registration) and `R/000-wrappers.R` (R wrappers; roxygen
  comments on the Rust items are copied here, so `@export` on a Rust doc comment is what makes the
  R function exported after `document()`).
- Rust errors returned as `savvy::Result::Err` become R errors via `handle_result`. Panics abort the
  R session in release builds (`panic = "abort"` in `Cargo.toml`), so never `unwrap()` on user input;
  convert `jiff::Error` to `savvy::Error` and return it.
- Savvy `#[savvy] struct`s become sealed R environments holding an external pointer
  (`R/000-wrappers.R`). The design for the Temporal types is instead columnar: plain R vectors
  (vctrs record types) in, plain vectors out, with Rust stateless. Prefer that over external pointers
  for anything user-facing (see `design.md` "Data model").

### R layer

- `R/000-wrappers.R` loads first so hand-written R files can override/extend generated functions.
- `R/zudate-package.R` holds the package-level roxygen block (`"_PACKAGE"`) and usethis namespace
  markers.
- Tests: `tests/testthat/`, edition 3. Rust tests live next to the Rust code.

## CI

- `.github/workflows/R-CMD-check.yaml` and `coverage.yaml` call reusable workflows from
  `pedrobtz/r-actions@v1` with `rust: true`. On pull requests the check runs the `quick` profile; a
  push to `main` or the `full-ci` label on a PR runs the `full` matrix. The warning
  "Rust compilation" is allow-listed; any other `R CMD check` WARNING fails CI.
- The coverage workflow commits `.github/badges/coverage.svg` to `main`; do not edit that file.
- `pkgdown.yaml` deploys the site to the `gh-pages` branch (`_pkgdown.yml`, Bootstrap 5).

## Conventions and gotchas

- Crate name and R package name are both `zudate`; the static lib must be `libzudate.a` and the init
  symbol `R_init_zudate`. Renaming either requires touching Makevars, `init.c`, the `.def` file and
  `Cargo.toml` together.
- `src/rust/.cargo/config.toml` exists only so `cargo test` links on Windows MSVC; it is excluded
  from the tarball on purpose.
- `cargo test` warns about an unexpected `savvy-test` cfg; that is expected (the feature is injected
  by `savvy-cli test`), do not "fix" it by removing the test module.
- CRAN installs with at most 2 CPUs: the vendored build must pass `-j 2` to cargo (see roadmap).
- Dates in `NEWS.md` / `DESCRIPTION` follow standard R package conventions; bump `Version:` in
  `DESCRIPTION` and `version` in `src/rust/Cargo.toml` together.
