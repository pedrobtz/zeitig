# zeitig: 'Temporal' Date and Time Types Backed by 'jiff'

Implements the 'Temporal' date and time model of Ecma Technical
Committee 39 (TC39) <https://tc39.es/proposal-temporal/docs/> (instants,
plain dates, plain times, plain date-times, zoned date-times and
durations) as vectorised R classes. All calendar and time zone
arithmetic, parsing and formatting is delegated to the 'Rust' crate
'jiff' <https://docs.rs/jiff>, which is bundled with the package, giving
nanosecond precision, Internet Extended Date/Time Format (IXDTF, RFC
9557) string support and time-zone-aware arithmetic with 'Temporal'
semantics.

## See also

Useful links:

- <https://pedrobtz.github.io/zeitig/>

- <https://github.com/pedrobtz/zeitig>

- Report bugs at <https://github.com/pedrobtz/zeitig/issues>

## Author

**Maintainer**: Pedro Baltazar <pedrobtz@gmail.com> \[copyright holder\]

Authors:

- Pedro Baltazar <pedrobtz@gmail.com> \[copyright holder\]

Other contributors:

- Andrew Gallant (Author of the bundled 'jiff' Rust crates)
  \[contributor, copyright holder\]

- Hiroaki Yutani (Author of the bundled 'savvy' Rust crates)
  \[contributor, copyright holder\]

- The authors of the dependency Rust crates (see inst/AUTHORS for
  details) \[copyright holder\]
