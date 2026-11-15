# ACME normal W0 — train-partition feedback read before the W1 revision

Stored endpoints of the **train** runs (interval-censored event times in seconds since the time origin, cause, original label) next to the W0 release's cumulative incidences from the time origin. Produced by `wsp-dev acme-table`; no fit. Of validation only the aggregate comparison below was seen.

```text
run(tail)        grp      fuel    p_atm  mdot   xO2   xF    Tad   Zst | cens      t_act  t_term cause                 label | model: P_ext@10/20/45/90  P_op@20/45/90  t50any
e_normal:21106G9 21-04-16 butane    0.50   0.80  0.207  0.500   2223  0.087 | interval    13.5   14.5 extinction            SE    | 0.08 0.27 0.62 0.66  0.01 0.22 0.34  30.0
e_normal:21106G7 21-04-16 butane    0.50   0.80  0.209  0.500   2228  0.087 | interval    19.5   20.5 extinction            SE    | 0.08 0.27 0.62 0.65  0.01 0.22 0.35  30.0
e_normal:21106G6 21-04-16 butane    0.50   0.80  0.210  0.500   2233  0.088 | interval    19.5   20.5 extinction            SE    | 0.08 0.27 0.61 0.65  0.01 0.23 0.35  30.0
e_normal:21106F3 21-04-16 butane    0.50   0.50  0.354  0.500   2635  0.138 | interval    12.5   13.5 operator_termination  FT    | 0.01 0.05 0.15 0.16  0.02 0.44 0.84  45.0
e_normal:21106F1 21-04-16 butane    0.50   0.80  0.359  0.500   2642  0.139 | interval    71.5   72.5 operator_termination  FT    | 0.03 0.10 0.28 0.30  0.02 0.38 0.69  45.0
e_normal:21106D1 21-04-16 butane    0.50   1.20  0.369  0.500   2659  0.142 | interval   119.5  120.5 extinction            SE    | 0.05 0.17 0.45 0.48  0.02 0.30 0.52  45.0
e_normal:21106C1 21-04-16 butane    0.50   1.30  0.377  0.125   2482  0.333 | interval    62.5   63.5 extinction            SE    | 0.09 0.29 0.65 0.69  0.01 0.20 0.31  30.0
e_normal:21106B1 21-04-16 butane    0.50   1.60  0.385  0.500   2682  0.147 | interval    61.5   62.5 extinction            SE    | 0.07 0.25 0.58 0.62  0.01 0.24 0.38  45.0
e_normal:21106A4 21-04-16 butane    0.50   1.60  0.393  0.250   2629  0.225 | interval    59.5   60.5 extinction            SE    | 0.09 0.28 0.63 0.67  0.01 0.21 0.33  30.0
e_normal:21125D1 21-05-05 butane    1.00   0.50  0.211  0.045   1701  0.423 | interval     0.5    1.5 extinction            SE    | 0.21 0.58 0.91 0.92  0.01 0.07 0.08  20.0
e_normal:21125D2 21-05-05 butane    1.00   0.50  0.211  0.045   1701  0.423 | interval     0.5    1.5 extinction            SE    | 0.21 0.58 0.91 0.92  0.01 0.07 0.08  20.0
e_normal:21291J3 21-10-18 butane    1.00   1.00  0.381  0.150   2530  0.301 | interval    82.5   83.5 extinction            SE    | 0.07 0.25 0.58 0.62  0.01 0.24 0.38  45.0
e_normal:21291J2 21-10-18 butane    1.00   1.50  0.383  0.150   2532  0.302 | interval    26.5   27.5 extinction            SE    | 0.14 0.42 0.80 0.82  0.01 0.13 0.18  30.0
e_normal:21125K3 21-05-05 butane    1.00   0.75  0.384  0.031   1785  0.651 | interval   124.5  125.5 operator_termination  FT    | 0.30 0.73 0.97 0.97  0.01 0.03 0.03  15.0
e_normal:21291J1 21-10-18 butane    1.00   0.75  0.385  0.150   2536  0.303 | interval    79.5   80.5 operator_termination  FT    | 0.05 0.16 0.42 0.46  0.02 0.32 0.54  45.0
e_normal:21291G2 21-10-18 butane    1.00   1.25  0.388  0.300   2644  0.200 | interval    59.5   60.5 extinction            SE    | 0.08 0.26 0.60 0.64  0.01 0.23 0.36  30.0
e_normal:21125K2 21-05-05 butane    1.00   3.00  0.389  0.031   1790  0.654 | interval    16.5   17.5 extinction            SE    | 0.96 1.00 1.00 1.00  0.00 0.00 0.00  5.0
e_normal:21125K1 21-05-05 butane    1.00   1.50  0.394  0.031   1795  0.657 | interval    34.5   35.5 extinction            SE    | 0.65 0.98 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21125A2 21-05-05 butane    1.00   0.50  0.395  0.031   1795  0.657 | interval     7.5    8.5 extinction            SE    | 0.17 0.49 0.86 0.88  0.01 0.10 0.12  30.0
e_normal:21291G1 21-10-18 butane    1.00   1.00  0.395  0.300   2653  0.202 | interval   160.5  161.5 extinction            SE    | 0.05 0.19 0.47 0.50  0.02 0.29 0.50  45.0
e_normal:21291D1 21-10-18 butane    1.00   1.50  0.398  0.300   2657  0.203 | interval    46.5   47.5 extinction            SE    | 0.10 0.32 0.69 0.72  0.01 0.19 0.28  30.0
e_normal:21291H2 21-10-18 butane    2.00   0.75  0.386  0.150   2537  0.304 | interval   157.5  158.5 operator_termination  FT    | 0.07 0.22 0.54 0.57  0.01 0.26 0.43  45.0
e_normal:21291H1 21-10-18 butane    2.00   1.25  0.387  0.150   2538  0.304 | interval    34.5   35.5 extinction            SE    | 0.14 0.43 0.81 0.83  0.01 0.12 0.17  30.0
e_normal:21291B1 21-10-18 butane    2.00   1.75  0.388  0.300   2644  0.200 | interval    31.5   32.5 extinction            SE    | 0.18 0.52 0.88 0.89  0.01 0.09 0.11  20.0
e_normal:21291C2 21-10-18 butane    2.00   1.75  0.389  0.300   2646  0.200 | interval    26.5   27.5 extinction            SE    | 0.18 0.52 0.88 0.89  0.01 0.09 0.11  20.0
e_normal:21291C1 21-10-18 butane    2.00   1.00  0.394  0.300   2652  0.202 | interval   245.5  246.5 operator_termination  FT    | 0.08 0.25 0.59 0.62  0.01 0.24 0.38  30.0
e_normal:21291A1 21-10-18 butane    2.00   1.50  0.399  0.300   2659  0.204 | interval   166.5  167.5 operator_termination  FT    | 0.14 0.42 0.80 0.83  0.01 0.13 0.17  30.0
e_normal:21153A5 21-06-02 ethane    0.50   0.75  0.332  0.300   2483  0.236 | interval   273.5  274.5 operator_termination  FT    | 0.04 0.13 0.36 0.39  0.02 0.34 0.61  45.0
e_normal:21153A4 21-06-02 ethane    0.50   0.50  0.335  0.300   2488  0.237 | interval    55.5   56.5 operator_termination  FT    | 0.02 0.07 0.21 0.23  0.02 0.41 0.77  45.0
e_normal:21153F1 21-06-02 ethane    0.50   1.50  0.341  0.300   2499  0.240 | interval    43.5   44.5 extinction            SE    | 0.11 0.34 0.72 0.75  0.01 0.17 0.25  30.0
e_normal:21153A3 21-06-02 ethane    0.50   1.00  0.364  0.300   2537  0.252 | interval   257.5  258.5 extinction            SE    | 0.05 0.18 0.46 0.49  0.02 0.30 0.51  45.0
e_normal:21153A2 21-06-02 ethane    0.50   1.50  0.376  0.300   2555  0.258 | interval    87.5   88.5 extinction            SE    | 0.09 0.30 0.67 0.70  0.01 0.20 0.30  30.0
e_normal:21153A1 21-06-02 ethane    0.50   2.36  0.389  0.300   2573  0.264 | interval    58.5   59.5 extinction            SE    | 0.17 0.51 0.87 0.89  0.01 0.09 0.11  20.0
e_normal:21088H2 21-03-29 ethane    0.93   1.00  0.365  0.500   2616  0.170 | interval   126.5  127.5 operator_termination  FT    | 0.06 0.20 0.49 0.53  0.02 0.28 0.47  45.0
e_normal:21088K1 21-03-29 ethane    0.93   1.00  0.370  0.250   2507  0.290 | interval    97.5   98.5 operator_termination  FT    | 0.08 0.25 0.59 0.63  0.01 0.23 0.37  30.0
e_normal:21088J1 21-03-29 ethane    0.93   2.00  0.373  0.250   2512  0.292 | interval    36.5   37.5 extinction            SE    | 0.21 0.58 0.92 0.93  0.01 0.06 0.07  20.0
e_normal:21088H1 21-03-29 ethane    0.93   1.00  0.376  0.500   2634  0.175 | interval    61.5   62.5 operator_termination  FT    | 0.06 0.19 0.47 0.51  0.02 0.29 0.49  45.0
e_normal:21088F1 21-03-29 ethane    0.93   0.50  0.380  1.000   2700  0.100 | interval    27.5   28.5 operator_termination  FT    | 0.02 0.06 0.17 0.18  0.02 0.43 0.82  45.0
e_normal:21088G1 21-03-29 ethane    0.93   2.00  0.380  0.500   2639  0.176 | interval    41.5   42.5 extinction            SE    | 0.15 0.46 0.84 0.86  0.01 0.11 0.14  30.0
e_normal:21088D1 21-03-29 ethane    0.93   1.00  0.383  1.000   2704  0.100 | interval    60.5   61.5 operator_termination  FT    | 0.05 0.16 0.42 0.45  0.02 0.32 0.55  45.0
e_normal:21088B1 21-03-29 ethane    0.93   2.00  0.390  1.000   2714  0.102 | interval    42.5   43.5 extinction            SE    | 0.13 0.40 0.78 0.81  0.01 0.14 0.19  30.0
e_normal:21088A1 21-03-29 ethane    0.93   2.00  0.394  1.000   2719  0.103 | interval    49.5   50.5 extinction            SE    | 0.13 0.40 0.78 0.80  0.01 0.14 0.20  30.0
e_normal:21088P1 21-03-29 ethane    1.00   1.00  0.180  0.500   2040  0.094 | interval     7.5    8.5 extinction            SE    | 0.25 0.65 0.94 0.95  0.01 0.04 0.05  20.0
e_normal:21088N1 21-03-29 ethane    1.00   2.00  0.180  0.500   2043  0.094 | interval     6.5    7.5 extinction            SE    | 0.58 0.96 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21162F2 21-06-11 ethane    1.00   1.50  0.207  0.300   2109  0.164 | interval     9.5   10.5 extinction            SE    | 0.36 0.81 0.98 0.98  0.01 0.02 0.02  15.0
e_normal:21162D1 21-06-11 ethane    1.00   0.50  0.208  0.300   2110  0.164 | interval    15.5   16.5 extinction            SE    | 0.07 0.25 0.59 0.62  0.01 0.24 0.38  30.0
e_normal:21162C5 21-06-11 ethane    1.00   2.36  0.208  0.300   2114  0.165 | interval     8.5    9.5 extinction            SE    | 0.60 0.97 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21162C3 21-06-11 ethane    1.00   1.00  0.209  0.300   2118  0.165 | interval     8.5    9.5 extinction            SE    | 0.21 0.57 0.91 0.92  0.01 0.07 0.08  20.0
e_normal:21162C4 21-06-11 ethane    1.00   1.50  0.209  0.300   2116  0.165 | interval     8.5    9.5 extinction            SE    | 0.36 0.81 0.98 0.98  0.01 0.02 0.02  15.0
e_normal:21162C1 21-06-11 ethane    1.00   0.50  0.210  0.300   2121  0.166 | interval    18.5   19.5 extinction            SE    | 0.07 0.24 0.58 0.61  0.01 0.24 0.39  45.0
e_normal:21162C2 21-06-11 ethane    1.00   0.75  0.210  0.300   2119  0.165 | interval    10.5   11.5 extinction            SE    | 0.14 0.42 0.80 0.82  0.01 0.13 0.18  30.0
e_normal:21088M4 21-03-29 ethane    1.00   1.00  0.210  1.000   2257  0.059 | interval    12.5   13.5 extinction            SE    | 0.15 0.45 0.83 0.85  0.01 0.12 0.15  30.0
e_normal:21088L1 21-03-29 ethane    1.00   2.00  0.210  1.000   2254  0.059 | interval     8.5    9.5 extinction            SE    | 0.39 0.83 0.99 0.99  0.01 0.01 0.01  15.0
e_normal:21162A4 21-06-11 ethane    1.00   0.85  0.372  0.300   2550  0.256 | interval   303.5  304.5 extinction            SE    | 0.06 0.19 0.47 0.51  0.02 0.29 0.49  45.0
e_normal:21162B1 21-06-11 ethane    1.00   1.00  0.379  0.300   2560  0.259 | interval   157.5  158.5 extinction            SE    | 0.07 0.23 0.56 0.59  0.01 0.25 0.41  45.0
e_normal:21162A3 21-06-11 ethane    1.00   1.00  0.390  0.300   2575  0.265 | interval   231.5  232.5 extinction            SE    | 0.07 0.22 0.54 0.58  0.01 0.26 0.42  45.0
e_normal:21162A2 21-06-11 ethane    1.00   1.50  0.395  0.300   2582  0.267 | interval    70.5   71.5 extinction            SE    | 0.12 0.38 0.76 0.79  0.01 0.15 0.21  30.0
e_normal:21162A1 21-06-11 ethane    1.00   2.36  0.400  0.300   2588  0.269 | interval    42.5   43.5 extinction            SE    | 0.23 0.62 0.93 0.94  0.01 0.05 0.06  20.0
e_normal:21153C7 21-06-02 ethane    2.00   1.00  0.382  0.300   2564  0.261 | interval    60.5   61.5 operator_termination  FT    | 0.10 0.31 0.67 0.70  0.01 0.20 0.29  30.0
e_normal:21153C5 21-06-02 ethane    2.00   0.50  0.384  0.300   2567  0.262 | interval    48.5   49.5 operator_termination  FT    | 0.03 0.11 0.31 0.34  0.02 0.36 0.66  45.0
e_normal:21153C6 21-06-02 ethane    2.00   1.50  0.384  0.300   2566  0.261 | interval    43.5   44.5 extinction            SE    | 0.17 0.50 0.87 0.89  0.01 0.09 0.11  20.0
e_normal:21153C4 21-06-02 ethane    2.00   0.75  0.389  0.300   2573  0.264 | interval   275.5  276.5 operator_termination  FT    | 0.06 0.20 0.50 0.54  0.02 0.28 0.46  45.0
e_normal:21153G1 21-06-02 ethane    2.00   1.00  0.392  0.300   2578  0.265 | interval   130.5  131.5 extinction            SE    | 0.09 0.30 0.66 0.69  0.01 0.20 0.31  30.0
e_normal:21153C3 21-06-02 ethane    2.00   1.00  0.395  0.300   2581  0.267 | interval   122.5  123.5 extinction            SE    | 0.09 0.30 0.66 0.69  0.01 0.20 0.31  30.0
e_normal:21153C2 21-06-02 ethane    2.00   1.50  0.397  0.300   2584  0.268 | interval    50.5   51.5 extinction            SE    | 0.17 0.49 0.86 0.88  0.01 0.10 0.12  30.0
e_normal:21153C1 21-06-02 ethane    2.00   2.36  0.399  0.300   2586  0.269 | interval    33.5   34.5 extinction            SE    | 0.31 0.74 0.97 0.97  0.01 0.03 0.03  15.0
e_normal:21130M3 21-05-10 propane   2.00   0.50  0.210  0.300   2169  0.138 | interval     4.5    5.5 extinction            SE    | 0.09 0.29 0.65 0.69  0.01 0.20 0.31  30.0
e_normal:21130A9 21-05-10 propane   2.00   0.25  0.211  0.047   1602  0.470 | interval     0.5    1.5 extinction            SE    | 0.13 0.41 0.79 0.81  0.01 0.14 0.19  30.0
e_normal:21130A1 21-05-10 propane   2.00   0.50  0.211  0.047   1602  0.470 | interval     0.5    1.5 extinction            SE    | 0.35 0.79 0.98 0.98  0.01 0.02 0.02  15.0
e_normal:21130A2 21-05-10 propane   2.00   1.00  0.211  0.047   1602  0.470 | interval     0.5    1.5 extinction            SE    | 0.73 0.99 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21130A5 21-05-10 propane   2.00   1.00  0.211  0.047   1602  0.470 | interval     0.5    1.5 extinction            SE    | 0.73 0.99 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21130M1 21-05-10 propane   2.00   1.00  0.211  0.300   2171  0.138 | interval     5.5    6.5 extinction            SE    | 0.25 0.65 0.94 0.95  0.01 0.04 0.05  20.0
e_normal:21130M2 21-05-10 propane   2.00   2.00  0.211  0.300   2170  0.138 | interval     6.5    7.5 extinction            SE    | 0.58 0.96 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21130N3 21-05-10 propane   3.00   0.50  0.216  0.300   2197  0.141 | interval     4.5    5.5 extinction            SE    | 0.10 0.33 0.70 0.73  0.01 0.18 0.27  30.0
e_normal:21130N4 21-05-10 propane   3.00   0.50  0.216  0.300   2197  0.141 | interval     4.5    5.5 extinction            SE    | 0.10 0.33 0.70 0.73  0.01 0.18 0.27  30.0
e_normal:21130G7 21-05-10 propane   3.00   0.12  0.217  0.047   1618  0.477 | interval     0.5    1.5 extinction            SE    | 0.05 0.18 0.47 0.50  0.02 0.30 0.50  45.0
e_normal:21130G6 21-05-10 propane   3.00   0.25  0.217  0.047   1618  0.477 | interval     0.5    1.5 extinction            SE    | 0.15 0.46 0.84 0.85  0.01 0.11 0.14  30.0
e_normal:21130G2 21-05-10 propane   3.00   0.50  0.217  0.047   1618  0.477 | interval     0.5    1.5 extinction            SE    | 0.40 0.84 0.99 0.99  0.01 0.01 0.01  15.0
e_normal:21130G3 21-05-10 propane   3.00   1.00  0.217  0.047   1618  0.477 | interval     0.5    1.5 extinction            SE    | 0.78 0.99 1.00 1.00  0.00 0.00 0.00  10.0
e_normal:21130N1 21-05-10 propane   3.00   1.00  0.217  0.300   2198  0.141 | interval     5.5    6.5 extinction            SE    | 0.28 0.70 0.96 0.96  0.01 0.03 0.04  20.0
e_normal:21130N2 21-05-10 propane   3.00   2.00  0.217  0.300   2198  0.141 | interval     5.5    6.5 extinction            SE    | 0.63 0.97 1.00 1.00  0.00 0.00 0.00  10.0
```

## Validation, aggregate only (sealed comparison of W0 with the fitted baselines)

```text
partition validation, common cases 84, excluded incomplete 0, reference constant
event block keys: ['any_event', 'causes', 'estimator', 'horizons']

integrated Brier-type entries:
  causes.extinction.integrated_brier                          hazard_fitted=0.1873  WSP_W0=0.2192  piecewise=0.2168  constant=0.2137  linear=0.1854
  causes.operator_termination.integrated_brier                hazard_fitted=0.0933  WSP_W0=0.1250  piecewise=0.1001  constant=0.0993  linear=0.0928
  causes.unresolved_support.integrated_brier                  hazard_fitted=0.0000  WSP_W0=0.0000  piecewise=0.0000  constant=0.0000  linear=0.0000

per-case metric means (group-equal):
  ibs_contrib[extinction]                 hazard_fitted=0.1885  WSP_W0=0.2219  piecewise=0.2167  constant=0.2159  linear=0.1853
  ibs_contrib[operator_termination]       hazard_fitted=0.0836  WSP_W0=0.1160  piecewise=0.0894  constant=0.0891  linear=0.0822
  ibs_contrib[unresolved_support]         hazard_fitted=0.0000  WSP_W0=0.0000  piecewise=0.0000  constant=0.0000  linear=0.0000

paired vs reference:
  hazard_fitted              ibs_contrib[extinction]            -0.0274 [-0.0350, -0.0199] cases 84 clusters 2
  hazard_fitted              ibs_contrib[operator_termination]  -0.0055 [-0.0089, -0.0021] cases 84 clusters 2
  hazard_fitted              ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 84 clusters 2
  WSP_W0                     ibs_contrib[extinction]            +0.0060 [+0.0024, +0.0096] cases 84 clusters 2
  WSP_W0                     ibs_contrib[operator_termination]  +0.0269 [+0.0187, +0.0351] cases 84 clusters 2
  WSP_W0                     ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 84 clusters 2
  piecewise                  ibs_contrib[extinction]            +0.0008 [-0.0155, +0.0171] cases 84 clusters 2
  piecewise                  ibs_contrib[operator_termination]  +0.0003 [-0.0030, +0.0036] cases 84 clusters 2
  piecewise                  ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 84 clusters 2
  linear                     ibs_contrib[extinction]            -0.0307 [-0.0473, -0.0141] cases 84 clusters 2
  linear                     ibs_contrib[operator_termination]  -0.0069 [-0.0095, -0.0044] cases 84 clusters 2
  linear                     ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 84 clusters 2
```
