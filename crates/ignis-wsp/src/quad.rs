//! Fixed numerical constants of the strict forwards: the standard-normal quantile function and a
//! seven-point Gauss–Hermite rule. Nothing here depends on data or on a release.

/// Probabilists' Gauss–Hermite nodes and weights for a standard normal variable (n = 7): for a
/// polynomial `f` of degree <= 13, `E[f(Z)] = sum_k W[k] f(Z[k])`. The weights sum to one.
pub const GH7_Z: [f64; 7] = [
    -3.750_439_717_725_742,
    -2.366_759_410_734_541,
    -1.154_405_394_739_968,
    0.0,
    1.154_405_394_739_968,
    2.366_759_410_734_541,
    3.750_439_717_725_742,
];
pub const GH7_W: [f64; 7] = [
    5.482_688_559_722_184e-4,
    3.075_712_396_758_645e-2,
    2.401_231_786_050_127e-1,
    4.571_428_571_428_571e-1,
    2.401_231_786_050_127e-1,
    3.075_712_396_758_645e-2,
    5.482_688_559_722_184e-4,
];

/// Standard normal quantile (Acklam's rational approximation, |relative error| < 1.2e-9), for
/// `p` in (0, 1).
pub fn normal_quantile(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const LOW: f64 = 0.024_25;
    assert!(p > 0.0 && p < 1.0, "normal_quantile needs p in (0, 1)");
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if p < LOW {
        tail((-2.0 * p.ln()).sqrt())
    } else if p > 1.0 - LOW {
        -tail((-2.0 * (1.0 - p).ln()).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// Standard normal CDF.
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * libm::erfc(-x / std::f64::consts::SQRT_2)
}
