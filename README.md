# General
This program is used to get quick, and simple, statistical information on a data set. Specifically, it is desiged to work on columnar data such a series of numbers separated by a new line.

In files which have multiple columns delimited by any number of spaces such as the following:

```
...
string0   5  1.0  10s  stringA 
string1   4  2.1  13s stringB
string2   3  3.2  16s stringC 
string3   2  4.3   4s  stringD
... 
```

The output, by default, for a normally distributed data set will look similar to the following:

```
Samples:        5000
Max:           86.00
Min:            7.90
Mean:          50.04
SD:             9.99
Outlier(s) >= 79.99
Outlier(s) <= 20.08
Percentile  bucket      count
100.00       86.9          1 
 99.98       85.9          2 
 99.94       83.9          1 
 99.92       81.9          1 
 99.90       80.9          2 
 99.86       79.9          3 
 99.80       78.9          4 
 99.72       77.9          8 
 99.56       76.9         11 -
 99.34       75.9          9 -
 99.16       74.9          9 -
 98.98       73.9         14 -
 98.70       72.9         14 -
 98.42       71.9         15 -
 98.12       70.9         25 ---
 97.62       69.9         26 ---
 97.10       68.9         44 -----
 96.22       67.9         39 -----
 95.44       66.9         49 ------
 94.46       65.9         67 --------
 93.12       64.9         58 -------
 91.96       63.9         86 -----------
 90.24       62.9         89 -----------
 88.46       61.9         96 ------------
 86.54       60.9        128 ----------------
 83.98       59.9        131 -----------------
 81.36       58.9        130 -----------------
 78.76       57.9        162 ---------------------
 75.52       56.9        164 ---------------------
 72.24       55.9        160 ---------------------
 69.04       54.9        175 -----------------------
 65.54       53.9        182 ------------------------
 61.90       52.9        187 ------------------------
 58.16       51.9        234 -------------------------------
 53.48       50.9        199 --------------------------
 49.50       49.9        192 -------------------------
 45.66       48.9        190 -------------------------
 41.86       47.9        181 ------------------------
 38.24       46.9        181 ------------------------
 34.62       45.9        174 -----------------------
 31.14       44.9        181 ------------------------
 27.52       43.9        181 ------------------------
 23.90       42.9        163 ---------------------
 20.64       41.9        141 ------------------
 17.82       40.9        111 --------------
 15.60       39.9        116 ---------------
 13.28       38.9        101 -------------
 11.26       37.9        107 --------------
  9.12       36.9         82 ----------
  7.48       35.9         81 ----------
  5.86       34.9         46 ------
  4.94       33.9         44 -----
  4.06       32.9         38 ----
  3.30       31.9         29 ---
  2.72       30.9         32 ----
  2.08       29.9         20 --
  1.68       28.9         15 -
  1.38       27.9         12 -
  1.14       26.9         11 -
  0.92       25.9         14 -
  0.64       24.9          6 
  0.52       23.9          7 
  0.38       22.9          2 
  0.34       21.9          6 
  0.22       20.9          4 
  0.14       19.9          3 
  0.08       18.9          1 
  0.06       17.9          1 
  0.04       14.9          1 
  0.02        7.9          1 
```

There are three sections to this output that can be turned off via commandline: 'info', 'percentiles', and 'bars'. The 'info' section is information related from `Samples` to `Percentile Bucket Count`. The 'percentiles' section is the numerical data in the three columns. And the 'bars' section is the dashed bars to the right of count.

The 'percentile' section has a default output limit of 100 lines. Though this is modifiable via commandline.

# Usage
The program is designed to be invoked in two distinct ways. First is by providing an input file and a column (zero indexed). This will be something like:

`qhist -c 1 -i /path/to/columnar_data`

When the data containts floating point values, such as column two in the example at the start of this readme, the `-s/--sig-figs` argument *must* be provided with the desired precision. 

`qhist --sig-figs 1 -c 1 -i /path/to/columnar_data`

The second method is using `stdin`. 

`cat /path/to_columnar_data | qhist --sig-figs 1 -c 1`

This is especially useful if the data contains time or temperature like data with units.

`cat /path/to/columnar_data | cut -c 19-20 | qhist`

Note that the default column is 0 so by applying `cut` to the data to remove the units we can run `qhist` over the time data in the above columnar data.




