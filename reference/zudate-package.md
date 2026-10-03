# zudate: 'Temporal' Date and Time Types Backed by 'jiff'

Implements the 'TC39 Temporal' date and time model
<https://tc39.es/proposal-temporal/docs/> (instants, plain dates, plain
times, plain date-times, zoned date-times and durations) as vectorised R
classes. All calendar and time zone arithmetic, parsing and formatting
is delegated to the 'Rust' crate 'jiff' <https://docs.rs/jiff>, which is
bundled with the package, giving nanosecond precision, 'RFC 9557' string
support and time-zone-aware arithmetic with 'Temporal' semantics.

## See also

Useful links:

- <https://pedrobtz.github.io/zudate/>

- <https://github.com/pedrobtz/zudate>

- Report bugs at <https://github.com/pedrobtz/zudate/issues>

## Author

**Maintainer**: Pedro Z <pedrobtz@gmail.com>

Authors:

- Pedro Z <pedrobtz@gmail.com>

Other contributors:

- The authors of the dependency Rust crates (see inst/AUTHORS for
  details) \[copyright holder\]
