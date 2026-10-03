# zudate roadmap

Goal: `zudate` 0.1.0 accepted on CRAN with the core Temporal types backed by vendored `jiff`.
Design decisions live in `design.md`; this file tracks the work and its order. Tick items as they
land and move anything that slips into "After 0.1.0" rather than letting 0.1.0 grow.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done.

## Milestone 0 - Skeleton verified (done)

- [x] savvy template builds and loads (`devtools::load_all()`), `R CMD check --as-cran` is clean.
- [x] CI: R-CMD-check (quick on PR, full on main), coverage badge, pkgdown deploy.
- [x] `CLAUDE.md`, `design.md`, `roadmap.md`.

## Milestone 1 - Build foundations (done)

Make the package CRAN-buildable before writing features, so every later PR is checked under the real
constraints.

- [x] Pin dependencies in `src/rust/Cargo.toml`: `jiff = { version = "0.2", default-features = false, features = ["std", "tz-system", "tz-fat", "tzdb-zoneinfo", "tzdb-bundle-platform", "perf-inline"] }`, `savvy = "0.11"`; edition 2021; commit `Cargo.lock`. (MSRV 1.81, from savvy.)
- [x] Release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true`, keep `panic = "abort"`.
- [x] `tools/vendor.sh`: `cargo vendor` -> `src/rust/vendor.tar.xz`; regenerate `inst/AUTHORS` and `LICENSE.note` from the vendored crates' metadata.
- [x] `src/Makevars.in` / `src/Makevars.win.in`: extract the tarball, write a cargo config pointing at `vendor/` (in a build-local `CARGO_HOME`), build with `--offline -j 2`, set `CARGO_HOME` inside the build tree; `cleanup`/`cleanup.win` remove extracted sources.
- [x] `configure.win`: pick `x86_64-pc-windows-gnu` or `aarch64-pc-windows-gnullvm` from the architecture.
- [x] `DESCRIPTION`: real title/description, authors (including the `cph` entry for crate authors), `SystemRequirements` with MSRV, `Imports: vctrs, rlang`, `Suggests: testthat, knitr, rmarkdown, pillar`.
- [x] Remove the template examples (`to_upper`, `int_times_int`, `Person`) once the first real function exists.
- [x] CI job: build the tarball, install it with networking disabled (`CARGO_NET_OFFLINE=true`, no `~/.cargo/registry`), run tests.
- [x] Measure tarball size; must stay under 5 MB. (1.5 MB; enforced by the offline-install job.)

## Milestone 2 - Core civil types (done)

`PlainDate`, `PlainTime`, `PlainDateTime` end to end; this establishes the patterns (record layout,
Rust column loops, option handling, error mapping) that the remaining types copy.

- [x] vctrs record classes + `format`/`print`/`vec_ptype2`/`vec_cast`/`vec_proxy_compare` for the three types (default record proxy; field order gives Temporal ordering).
- [x] Constructors from components and from RFC 9557 strings (Rust parser); `overflow = "constrain" | "reject"`.
- [x] Field accessors (`year()` ... `nanosecond()`, `day_of_week()`, `day_of_year()`, `week_of_year()`, `year_of_week()`, `days_in_month()`, `days_in_year()`, `in_leap_year()`), plus `days_in_week()`, `months_in_year()`, `temporal_fields()`.
- [x] `temporal_with()`, `with_plain_time()`, `with_plain_date()`.
- [x] Conversions among the three (`to_plain_date()`, `to_plain_time()`, `to_plain_date_time()`), and from/to `Date`, `POSIXlt`, `POSIXct` (`as_plain_*()` generics, `as.Date()`, `as.POSIXct()`/`as.POSIXlt()`).
- [x] Rust error -> `zudate_error` condition with element index (`zudate_range_error`; argument types -> `zudate_type_error`).
- [x] `temporal_compare()` / `temporal_equals()` for the civil types.
- [x] Tests from Temporal docs examples for each method; `NA` and zero-length inputs in every test file.

## Milestone 3 - Duration and arithmetic (done)

- [x] `zudate_duration` record class, constructor from components and ISO 8601 strings, uniform-sign and range validation.
- [x] `temporal_add()`/`temporal_subtract()` and `+`/`-` via `vec_arith` for Plain* types.
- [x] `temporal_until()`/`temporal_since()` with `largest_unit`, `smallest_unit`, `rounding_increment`, `rounding_mode`.
- [x] `temporal_round()` for Plain* types and durations (`relative_to` for calendar units).
- [x] `duration_total()`, `duration_compare()`, `abs()`, unary `-`, `sign()`, `duration_blank()`.
- [x] `difftime` conversions (`as_duration()`, `as_difftime()`).
- [x] CI job building with the MSRV (rustc 1.81) from the vendored crates.
- [x] Property tests in Rust: `a + (b - a) == b` for every type pair, round trip of strings (deterministic LCG, no extra crate).

## Milestone 4 - Instant, ZonedDateTime, Now

- [ ] `zudate_instant`: constructor from strings and epoch units, `epoch_seconds()`, `epoch_milliseconds()`, `epoch_nanoseconds()` (string), arithmetic with time-unit durations, `temporal_round()`, `until`/`since`.
- [ ] `zudate_zoned_date_time`: per-element time zone, constructor with `disambiguation` and `offset` options, parsing of `[Zone]` and `[!Zone]` annotations, `offset()`, `offset_nanoseconds()`, `time_zone()`, `hours_in_day()`, `start_of_day()`, `time_zone_transition()`, `with_time_zone()`, calendar-aware `add`/`until`/`round`.
- [ ] Per-call time zone cache in Rust; invalid identifiers produce `zudate_range_error`.
- [ ] `now_*()` functions; default zone from `Sys.timezone()`.
- [ ] `POSIXct` conversions both ways with documented precision loss.
- [ ] `temporal_equals()` vs `==` semantics as decided in `design.md` section 11.
- [ ] Tests covering DST gaps and overlaps in at least three zones, fixed offsets, `Etc/UTC`, and the `TZ`/`TZDIR` environment handling on all CI platforms.

## Milestone 5 - Formatting, parsing, polish

- [ ] `format()` options: `fractional_second_digits`, `smallest_unit`, `rounding_mode`, `offset`, `time_zone_name`, `calendar_name`.
- [ ] `temporal_strftime()` / `temporal_strptime()`.
- [ ] `pillar` methods (tibble columns), `str()` output, `summary()`.
- [ ] `as_*()` S3 generics complete for character, base classes and all zudate classes.
- [ ] `README.md` with install instructions and a worked example; `vignettes/zudate.Rmd` (overview and Temporal mapping) and `vignettes/time-zones.Rmd`.
- [ ] pkgdown reference grouped by Temporal type.
- [ ] Deviation table in `design.md` section 9 reviewed against the final behaviour.

## Milestone 6 - CRAN release 0.1.0

- [ ] `R CMD check --as-cran` clean on Linux, macOS (arm64 and x86_64), Windows (Rtools x86_64), plus `rhub` Rust images and win-builder (release and devel).
- [ ] Offline install from the tarball verified on each platform.
- [ ] Spelling (`spelling::spell_check_package()`), URLs (`urlchecker`), examples and tests under 60 s total.
- [ ] `NEWS.md` entry for 0.1.0; `Version: 0.1.0` in `DESCRIPTION` and `src/rust/Cargo.toml`.
- [ ] `cran-comments.md` noting: Rust package per CRAN policy, vendored crates, cargo/rustc versions reported by `configure`, tarball size, platforms tested.
- [ ] Submit; address CRAN feedback; tag `v0.1.0` and publish the GitHub release (triggers pkgdown).

## After 0.1.0

- `PlainYearMonth`, `PlainMonthDay` layered on `civil::Date`.
- Non-ISO calendars if `jiff` gains them, or via a separate calendar layer.
- `toLocaleString()` equivalent (ICU or `jiff` locale support).
- `SignedDuration` and `jiff`-specific extras (`Span::to_duration`, `Zoned::nth_weekday_of_month`) behind clearly non-Temporal names.
- Zero-copy integration with `clock` and `data.table` time types.
- WebAssembly/webR verification.
- Optional bundled tzdb on all platforms (`tzdb-bundle-always`) as a feature toggle for reproducible results independent of the OS database.
