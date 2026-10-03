# Summaries of Temporal vectors

[`summary()`](https://rdrr.io/r/base/summary.html) gives the minimum,
quartiles (as observed values, i.e. quantile type 1), maximum and the
number of missing values, in Temporal's ordering. Durations are ordered
by length with 24-hour days, so durations with years or months cannot be
summarised.

## Usage

``` r
# S3 method for class 'zudate_plain_date'
summary(object, ...)

# S3 method for class 'zudate_plain_time'
summary(object, ...)

# S3 method for class 'zudate_plain_date_time'
summary(object, ...)

# S3 method for class 'zudate_instant'
summary(object, ...)

# S3 method for class 'zudate_zoned_date_time'
summary(object, ...)

# S3 method for class 'zudate_duration'
summary(object, ...)
```

## Arguments

- object:

  A Temporal vector.

- ...:

  Not used.

## Value

A named character vector of class `table`.

## Examples

``` r
summary(plain_date(2020, 1:12, 1))
#>       Min.    1st Qu.     Median    3rd Qu.       Max. 
#> 2020-01-01 2020-03-01 2020-06-01 2020-09-01 2020-12-01 
summary(duration(hours = c(1, 5, NA)))
#>    Min. 1st Qu.  Median 3rd Qu.    Max.    NA's 
#>    PT1H    PT1H    PT1H    PT5H    PT5H       1 
```
