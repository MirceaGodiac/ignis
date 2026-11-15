# ACME inverse W0 — train-partition feedback read before the W1 revision

Stored endpoints of the **train** runs (interval-censored event times in seconds since the time origin, cause, original label) next to the W0 release's cumulative incidences from the time origin. Produced by `wsp-dev acme-table`; no fit. Of validation only the aggregate comparison below was seen.

```text
run(tail)        grp      fuel    p_atm  mdot   xO2   xF    Tad   Zst | cens      t_act  t_term cause                 label | model: P_ext@10/20/45/90  P_op@20/45/90  t50any
_inverse:21270C1 21-09-27 butane    1.09   5.37  0.500  0.169   2682  0.334 | interval    69.5   70.5 operator_termination  FT    | 0.07 0.24 0.58 0.61  0.01 0.24 0.39  45.0
_inverse:21270A3 21-09-27 butane    1.09   2.68  0.850  0.171   2868  0.447 | interval    34.5   35.5 operator_termination  FT    | 0.01 0.05 0.16 0.18  0.02 0.43 0.82  45.0
_inverse:21270H2 21-09-27 butane    1.09   2.68  0.850  0.139   2821  0.491 | interval    42.5   43.5 operator_termination  FT    | 0.02 0.06 0.18 0.20  0.02 0.42 0.80  45.0
_inverse:21270A4 21-09-27 butane    1.09   4.03  0.850  0.171   2868  0.447 | interval    89.5   90.5 operator_termination  FT    | 0.03 0.10 0.28 0.31  0.02 0.38 0.69  45.0
_inverse:21270H3 21-09-27 butane    1.09   4.03  0.850  0.139   2821  0.491 | interval    42.5   43.5 operator_termination  FT    | 0.03 0.12 0.32 0.35  0.02 0.36 0.65  45.0
_inverse:21270H5 21-09-27 butane    1.09   4.69  0.850  0.136   2815  0.496 | interval    45.5   46.5 extinction            SE    | 0.04 0.15 0.39 0.42  0.02 0.33 0.58  45.0
_inverse:21270A1 21-09-27 butane    1.09   5.37  0.850  0.175   2873  0.442 | interval    59.5   60.5 operator_termination  FT    | 0.04 0.15 0.41 0.44  0.02 0.32 0.56  45.0
_inverse:21270H1 21-09-27 butane    1.09   5.37  0.850  0.140   2823  0.489 | interval    50.5   51.5 extinction            SE    | 0.05 0.17 0.45 0.48  0.02 0.30 0.52  45.0
_inverse:21270A2 21-09-27 butane    1.09   8.95  0.850  0.174   2872  0.443 | interval    89.5   90.5 operator_termination  FT    | 0.10 0.32 0.69 0.72  0.01 0.19 0.28  30.0
_inverse:21270H4 21-09-27 butane    1.09   8.95  0.850  0.138   2820  0.492 | interval    82.5   83.5 operator_termination  FT    | 0.11 0.35 0.73 0.76  0.01 0.17 0.24  30.0
_inverse:21256F1 21-09-13 butane    2.18   5.37  0.210  0.088   1956  0.281 | interval    65.5   66.5 operator_termination  FT    | 0.52 0.93 0.99 0.99  0.00 0.01 0.01  10.0
_inverse:21256M1 21-09-13 butane    2.18   5.37  0.210  0.070   1879  0.326 | interval     0.5    1.5 extinction            SE    | 0.60 0.96 1.00 1.00  0.00 0.00 0.00  10.0
_inverse:21256C2 21-09-13 butane    2.18   1.79  0.500  0.087   2505  0.475 | interval     8.5    9.5 extinction            SE    | 0.03 0.10 0.29 0.32  0.02 0.38 0.68  45.0
_inverse:21256C1 21-09-13 butane    2.18   5.37  0.500  0.088   2509  0.472 | interval     8.5    9.5 extinction            SE    | 0.16 0.47 0.84 0.86  0.01 0.11 0.14  30.0
_inverse:21256K1 21-09-13 butane    2.18   5.37  0.500  0.070   2416  0.525 | interval     3.5    4.5 extinction            SE    | 0.20 0.55 0.90 0.91  0.01 0.07 0.09  20.0
_inverse:21256A4 21-09-13 butane    2.18   1.34  0.850  0.088   2678  0.592 | interval     4.5    5.5 extinction            SE    | 0.01 0.04 0.13 0.14  0.02 0.45 0.86  45.0
_inverse:21256H4 21-09-13 butane    2.18   1.34  0.850  0.070   2585  0.642 | interval     2.5    3.5 extinction            SE    | 0.01 0.05 0.16 0.18  0.02 0.43 0.82  45.0
_inverse:21256A3 21-09-13 butane    2.18   1.79  0.850  0.088   2678  0.592 | interval    10.5   11.5 extinction            SE    | 0.02 0.07 0.19 0.21  0.02 0.42 0.79  45.0
_inverse:21256H3 21-09-13 butane    2.18   1.79  0.850  0.070   2585  0.642 | interval     4.5    5.5 extinction            SE    | 0.02 0.08 0.24 0.27  0.02 0.40 0.73  45.0
_inverse:21256A2 21-09-13 butane    2.18   3.58  0.850  0.088   2678  0.592 | interval    10.5   11.5 extinction            SE    | 0.06 0.19 0.48 0.51  0.02 0.29 0.49  45.0
_inverse:21256H2 21-09-13 butane    2.18   3.58  0.850  0.070   2586  0.642 | interval     6.5    7.5 extinction            SE    | 0.07 0.23 0.56 0.59  0.01 0.25 0.41  45.0
_inverse:21256A1 21-09-13 butane    2.18   5.37  0.850  0.088   2679  0.592 | interval     8.5    9.5 extinction            SE    | 0.10 0.33 0.70 0.73  0.01 0.18 0.27  30.0
_inverse:21256B1 21-09-13 butane    2.18   5.37  0.850  0.087   2673  0.595 | interval    12.5   13.5 extinction            SE    | 0.10 0.33 0.71 0.74  0.01 0.18 0.26  30.0
_inverse:21256H1 21-09-13 butane    2.18   5.37  0.850  0.070   2586  0.642 | interval     6.5    7.5 extinction            SE    | 0.13 0.40 0.78 0.81  0.01 0.14 0.19  30.0
_inverse:21256J1 21-09-13 butane    2.18   5.37  0.850  0.070   2585  0.642 | interval     6.5    7.5 extinction            SE    | 0.13 0.40 0.78 0.81  0.01 0.14 0.19  30.0
_inverse:21256A5 21-09-13 butane    2.18   8.95  0.850  0.088   2678  0.592 | interval     8.5    9.5 extinction            SE    | 0.22 0.59 0.92 0.93  0.01 0.06 0.07  20.0
_inverse:21256H5 21-09-13 butane    2.18   8.95  0.850  0.070   2585  0.642 | interval     6.5    7.5 extinction            SE    | 0.27 0.69 0.96 0.96  0.01 0.04 0.04  20.0
_inverse:21196L2 21-07-15 ethane    0.49   2.79  0.210  0.285   2111  0.173 | interval    62.5   63.5 operator_termination  FT    | 0.08 0.26 0.60 0.64  0.01 0.23 0.36  30.0
_inverse:21196M2 21-07-15 ethane    0.49   2.79  0.300  0.279   2401  0.231 | interval    65.5   66.5 operator_termination  FT    | 0.04 0.13 0.35 0.38  0.02 0.35 0.62  45.0
nverse:21196M2_1 21-07-15 ethane    0.49   5.59  0.300  0.276   2398  0.233 | interval    63.5   64.5 operator_termination  FT    | 0.11 0.35 0.72 0.75  0.01 0.17 0.25  30.0
_inverse:21209K1 21-07-28 ethane    0.54   1.86  0.180  0.196   1885  0.206 | interval    25.5   26.5 operator_termination  FT    | 0.08 0.26 0.61 0.64  0.01 0.23 0.36  30.0
_inverse:21209F3 21-07-28 ethane    0.54   1.86  0.210  0.200   2030  0.228 | interval    65.5   66.5 operator_termination  FT    | 0.06 0.19 0.47 0.51  0.02 0.29 0.49  45.0
_inverse:21209F2 21-07-28 ethane    0.54   3.73  0.210  0.205   2036  0.224 | interval    65.5   66.5 operator_termination  FT    | 0.15 0.46 0.84 0.86  0.01 0.11 0.14  30.0
_inverse:21209F1 21-07-28 ethane    0.54   5.59  0.210  0.212   2045  0.218 | interval    65.5   66.5 operator_termination  FT    | 0.27 0.68 0.96 0.96  0.01 0.04 0.04  20.0
_inverse:21209J1 21-07-28 ethane    0.54   5.59  0.300  0.217   2340  0.277 | interval    65.5   66.5 operator_termination  FT    | 0.13 0.41 0.79 0.81  0.01 0.13 0.18  30.0
_inverse:21209H1 21-07-28 ethane    0.54   5.59  0.500  0.221   2621  0.380 | interval    65.5   66.5 operator_termination  FT    | 0.07 0.22 0.53 0.57  0.01 0.26 0.43  45.0
_inverse:21209A1 21-07-28 ethane    0.54   3.73  0.850  0.240   2821  0.479 | interval    25.5   26.5 operator_termination  FT    | 0.02 0.07 0.21 0.23  0.02 0.41 0.77  45.0
_inverse:21209B2 21-07-28 ethane    0.54   3.73  0.850  0.197   2763  0.527 | interval    47.5   48.5 operator_termination  FT    | 0.02 0.09 0.24 0.27  0.02 0.40 0.73  45.0
_inverse:21209A4 21-07-28 ethane    0.54   5.59  0.850  0.224   2802  0.495 | interval    65.5   66.5 operator_termination  FT    | 0.04 0.14 0.38 0.41  0.02 0.34 0.59  45.0
_inverse:21209A3 21-07-28 ethane    0.54   9.07  0.850  0.239   2820  0.480 | interval    66.5   67.5 operator_termination  FT    | 0.08 0.27 0.62 0.65  0.01 0.22 0.35  30.0
_inverse:21209B1 21-07-28 ethane    0.54   9.07  0.850  0.234   2814  0.485 | interval   135.5  136.5 operator_termination  FT    | 0.08 0.28 0.63 0.66  0.01 0.22 0.34  30.0
_inverse:21196A2 21-07-15 ethane    1.09   5.59  0.210  0.298   2120  0.167 | interval    34.5   35.5 operator_termination  FT    | 0.30 0.74 0.97 0.97  0.01 0.03 0.03  15.0
_inverse:21196F2 21-07-15 ethane    1.09   2.79  0.150  0.292   1785  0.128 | interval    65.5   66.5 operator_termination  FT    | 0.25 0.66 0.95 0.95  0.01 0.04 0.05  20.0
_inverse:21196F6 21-07-15 ethane    1.09   2.79  0.150  0.288   1783  0.129 | interval    65.5   66.5 operator_termination  FT    | 0.25 0.66 0.95 0.95  0.01 0.04 0.05  20.0
_inverse:21196F1 21-07-15 ethane    1.09   5.59  0.150  0.295   1787  0.127 | interval    35.5   36.5 operator_termination  FT    | 0.58 0.96 1.00 1.00  0.00 0.00 0.00  10.0
_inverse:21242P1 21-08-30 propane   0.54   5.44  0.300  0.428   2507  0.144 | interval    20.5   21.5 operator_termination  FT    | 0.08 0.28 0.63 0.66  0.01 0.22 0.34  30.0
_inverse:21242A1 21-08-30 propane   0.54   5.44  0.850  0.429   2978  0.306 | interval    11.5   12.5 operator_termination  FT    | 0.03 0.09 0.25 0.27  0.02 0.39 0.73  45.0
_inverse:21228L1 21-08-16 propane   1.09   3.63  0.210  0.087   1868  0.330 | interval     2.5    3.5 extinction            SE    | 0.30 0.73 0.97 0.97  0.01 0.03 0.03  15.0
_inverse:21228L2 21-08-16 propane   1.09   5.44  0.210  0.087   1867  0.331 | interval    15.5   16.5 operator_termination  FT    | 0.49 0.92 0.99 0.99  0.00 0.01 0.01  15.0
_inverse:21228K2 21-08-16 propane   1.09   1.81  0.500  0.087   2402  0.530 | interval    11.5   12.5 extinction            SE    | 0.03 0.10 0.28 0.30  0.02 0.38 0.70  45.0
_inverse:21228K1 21-08-16 propane   1.09   5.44  0.500  0.087   2403  0.530 | interval     8.5    9.5 extinction            SE    | 0.15 0.45 0.83 0.85  0.01 0.11 0.15  30.0
_inverse:21228J1 21-08-16 propane   1.09   1.81  0.850  0.087   2574  0.647 | interval    17.5   18.5 extinction            SE    | 0.02 0.06 0.19 0.20  0.02 0.42 0.80  45.0
_inverse:21228H2 21-08-16 propane   1.09   1.81  0.850  0.087   2574  0.646 | interval    14.5   15.5 extinction            SE    | 0.02 0.06 0.19 0.21  0.02 0.42 0.79  45.0
_inverse:21228H1 21-08-16 propane   1.09   5.44  0.850  0.087   2575  0.646 | interval     7.5    8.5 extinction            SE    | 0.10 0.32 0.69 0.72  0.01 0.19 0.28  30.0
_inverse:21228F1 21-08-16 propane   2.18   5.44  0.210  0.088   1873  0.327 | interval    64.5   65.5 operator_termination  FT    | 0.61 0.97 1.00 1.00  0.00 0.00 0.00  10.0
_inverse:21228C4 21-08-16 propane   2.18   1.81  0.500  0.088   2409  0.527 | interval     8.5    9.5 extinction            SE    | 0.04 0.13 0.36 0.39  0.02 0.34 0.61  45.0
_inverse:21228C3 21-08-16 propane   2.18   3.63  0.500  0.088   2410  0.527 | interval     6.5    7.5 extinction            SE    | 0.11 0.35 0.73 0.76  0.01 0.17 0.24  30.0
_inverse:21228C1 21-08-16 propane   2.18   5.44  0.500  0.089   2415  0.524 | interval     7.5    8.5 extinction            SE    | 0.20 0.56 0.91 0.92  0.01 0.07 0.08  20.0
_inverse:21228C2 21-08-16 propane   2.18   9.07  0.500  0.089   2414  0.524 | interval    65.5   66.5 operator_termination  FT    | 0.40 0.84 0.99 0.99  0.01 0.01 0.01  15.0
_inverse:21228A4 21-08-16 propane   2.18   1.81  0.850  0.089   2586  0.641 | interval    11.5   12.5 extinction            SE    | 0.02 0.09 0.24 0.27  0.02 0.40 0.73  45.0
_inverse:21228A3 21-08-16 propane   2.18   3.63  0.850  0.089   2586  0.641 | interval    11.5   12.5 extinction            SE    | 0.07 0.24 0.57 0.60  0.01 0.25 0.40  45.0
_inverse:21228A1 21-08-16 propane   2.18   5.44  0.850  0.090   2587  0.640 | interval     7.5    8.5 extinction            SE    | 0.13 0.41 0.79 0.81  0.01 0.14 0.19  30.0
_inverse:21228B1 21-08-16 propane   2.18   5.44  0.850  0.089   2586  0.641 | interval    15.5   16.5 extinction            SE    | 0.13 0.41 0.79 0.81  0.01 0.14 0.19  30.0
_inverse:21228A2 21-08-16 propane   2.18   9.07  0.850  0.089   2587  0.641 | interval    10.5   11.5 extinction            SE    | 0.27 0.69 0.96 0.96  0.01 0.04 0.04  20.0
_inverse:21222F1 21-08-10 propane   3.27   5.44  0.210  0.080   1835  0.349 | interval     0.5    1.5 extinction            SE    | 0.72 0.99 1.00 1.00  0.00 0.00 0.00  10.0
_inverse:21222C2 21-08-10 propane   3.27   3.63  0.500  0.080   2359  0.551 | interval     7.5    8.5 extinction            SE    | 0.15 0.46 0.84 0.85  0.01 0.11 0.14  30.0
_inverse:21222C1 21-08-10 propane   3.27   5.44  0.500  0.080   2359  0.551 | interval     5.5    6.5 extinction            SE    | 0.27 0.69 0.96 0.96  0.01 0.04 0.04  20.0
_inverse:21222A5 21-08-10 propane   3.27   1.36  0.850  0.080   2531  0.665 | interval     2.5    3.5 extinction            SE    | 0.02 0.08 0.22 0.25  0.02 0.41 0.75  45.0
_inverse:21222A4 21-08-10 propane   3.27   1.81  0.850  0.080   2531  0.665 | interval    12.5   13.5 extinction            SE    | 0.03 0.12 0.33 0.36  0.02 0.36 0.64  45.0
_inverse:21222A2 21-08-10 propane   3.27   3.63  0.850  0.080   2532  0.665 | interval    15.5   16.5 extinction            SE    | 0.10 0.32 0.69 0.72  0.01 0.19 0.28  30.0
_inverse:21222A1 21-08-10 propane   3.27   5.44  0.850  0.080   2533  0.665 | interval     9.5   10.5 extinction            SE    | 0.18 0.52 0.88 0.90  0.01 0.08 0.10  20.0
_inverse:21222B1 21-08-10 propane   3.27   5.44  0.850  0.080   2531  0.665 | interval    16.5   17.5 extinction            SE    | 0.18 0.52 0.88 0.90  0.01 0.08 0.10  20.0
_inverse:21222A3 21-08-10 propane   3.27   9.07  0.850  0.080   2532  0.665 | interval    17.5   18.5 extinction            SE    | 0.36 0.81 0.98 0.98  0.01 0.02 0.02  15.0
```

## Validation, aggregate only (sealed comparison of W0 with the fitted baselines)

```text
partition validation, common cases 49, excluded incomplete 5, reference constant
event block keys: ['any_event', 'causes', 'estimator', 'horizons']

integrated Brier-type entries:
  causes.extinction.integrated_brier                          piecewise=0.2512  hazard_fitted=0.1426  constant=0.2521  linear=0.1731  WSP_W0=0.3023
  causes.operator_termination.integrated_brier                piecewise=0.1265  hazard_fitted=0.1178  constant=0.1532  linear=0.1314  WSP_W0=0.1353
  causes.unresolved_support.integrated_brier                  piecewise=0.0000  hazard_fitted=0.0000  constant=0.0000  linear=0.0000  WSP_W0=0.0000

per-case metric means (group-equal):
  ibs_contrib[extinction]                 piecewise=0.2515  hazard_fitted=0.1270  constant=0.2450  linear=0.1524  WSP_W0=0.2925
  ibs_contrib[operator_termination]       piecewise=0.1293  hazard_fitted=0.1231  constant=0.1558  linear=0.1366  WSP_W0=0.1388
  ibs_contrib[unresolved_support]         piecewise=0.0000  hazard_fitted=0.0000  constant=0.0000  linear=0.0000  WSP_W0=0.0000

paired vs reference:
  piecewise                  ibs_contrib[extinction]            +0.0065 [-0.0454, +0.0583] cases 49 clusters 2
  piecewise                  ibs_contrib[operator_termination]  -0.0265 [-0.0277, -0.0254] cases 49 clusters 2
  piecewise                  ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 49 clusters 2
  hazard_fitted              ibs_contrib[extinction]            -0.1180 [-0.1777, -0.0583] cases 49 clusters 2
  hazard_fitted              ibs_contrib[operator_termination]  -0.0327 [-0.0520, -0.0135] cases 49 clusters 2
  hazard_fitted              ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 49 clusters 2
  linear                     ibs_contrib[extinction]            -0.0926 [-0.1873, +0.0021] cases 49 clusters 2
  linear                     ibs_contrib[operator_termination]  -0.0192 [-0.0379, -0.0005] cases 49 clusters 2
  linear                     ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 49 clusters 2
  WSP_W0                     ibs_contrib[extinction]            +0.0474 [+0.0288, +0.0661] cases 49 clusters 2
  WSP_W0                     ibs_contrib[operator_termination]  -0.0171 [-0.0228, -0.0113] cases 49 clusters 2
  WSP_W0                     ibs_contrib[unresolved_support]    +0.0000 [+0.0000, +0.0000] cases 49 clusters 2
```
