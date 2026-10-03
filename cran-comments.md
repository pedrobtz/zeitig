## Submission

Initial submission of zudate 0.1.0, an implementation of the TC39 Temporal
date/time model backed by the Rust crate 'jiff'.

## Rust

This package contains Rust code and follows the CRAN policy for Rust
packages (https://cran.r-project.org/web/packages/using_rust.html):

* All Rust dependencies are vendored in `src/rust/vendor.tar.xz` (1.5 MB,
  29 crates) and the build runs with `--offline`; no network access is
  needed. The tarball is extracted with R's own `untar()`.
* `configure` / `configure.win` report the versions of `cargo` and `rustc`.
* Cargo runs with `-j 2` and a `CARGO_HOME` inside the build directory, so
  nothing is written outside it; `cleanup` removes the extracted sources.
* The authors of the vendored crates are listed in `inst/AUTHORS` and
  credited in `Authors@R` as a `cph` entry; their licences (MIT, Apache-2.0,
  Unlicense, Unicode-3.0, all permissive) are summarised in `LICENSE.note`.
* `SystemRequirements` states the minimum Rust version (1.81), which is
  checked in CI.

## Test environments

* GitHub Actions: Ubuntu (R release, oldrel-1), macOS (arm64, R release),
  Windows (R release, Rtools); r-devel containers with clang 23 and GCC 16.
* Offline installation from the source tarball on Linux, macOS and Windows.
* TODO before submission: win-builder (release, devel) and R-hub Rust images.

## R CMD check results

0 errors | 0 warnings | 1 note

* This is a new release.
* The installed size may be reported as large (about 6 MB of `libs` on
  Linux): the shared library statically links the Rust standard library
  with its debug information, which R's `--strip` / CRAN's binary builds
  remove.
