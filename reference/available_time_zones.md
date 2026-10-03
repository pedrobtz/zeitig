# Available time zones

Lists the IANA time zone identifiers known to the time zone database
used by zudate. On Linux and macOS this is the system database
(`/usr/share/zoneinfo` or the directory in the `TZDIR` environment
variable); on Windows it is the copy of the IANA database bundled with
the package.

## Usage

``` r
available_time_zones()
```

## Value

A sorted character vector of time zone identifiers.

## Details

This is the equivalent of `Intl.supportedValuesOf("timeZone")` in
JavaScript, which is what Temporal uses to validate time zone
identifiers.

## Examples

``` r
head(available_time_zones())
#> [1] "Africa/Abidjan"     "Africa/Accra"       "Africa/Addis_Ababa"
#> [4] "Africa/Algiers"     "Africa/Asmara"      "Africa/Asmera"     
"Europe/Lisbon" %in% available_time_zones()
#> [1] TRUE
```
