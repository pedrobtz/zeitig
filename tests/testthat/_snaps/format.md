# summary() and str()

    Code
      summary(zoned_date_time(2020, 1, 1:3, time_zone = "UTC"))
    Output
                                Min.                        1st Qu. 
      2020-01-01T00:00:00+00:00[UTC] 2020-01-01T00:00:00+00:00[UTC] 
                              Median                        3rd Qu. 
      2020-01-02T00:00:00+00:00[UTC] 2020-01-03T00:00:00+00:00[UTC] 
                                Max. 
      2020-01-03T00:00:00+00:00[UTC] 
    Code
      str(plain_date(2020, 1:3, 1))
    Output
       pdate [1:3] 2020-01-01, 2020-02-01, 2020-03-01

