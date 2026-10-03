# printing

    Code
      plain_date(c("2020-01-01", NA))
    Output
      <plain_date[2]>
      [1] 2020-01-01 <NA>      
    Code
      plain_time("12:30:00.5")
    Output
      <plain_time[1]>
      [1] 12:30:00.5
    Code
      plain_date_time("2020-01-01T12:30")
    Output
      <plain_date_time[1]>
      [1] 2020-01-01T12:30:00
    Code
      data.frame(d = plain_date(2020, 1:2, 1), t = plain_time(1:2))
    Output
                 d        t
      1 2020-01-01 01:00:00
      2 2020-02-01 02:00:00

