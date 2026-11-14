//! Development-only synthetic expert. NOT a scientific model.
//!
//! Rust port of the former Python `ignis.inference.fixture_expert.SyntheticD2Expert`. It exists so
//! the runtime and the UI can be exercised end to end on the synthetic `/fixtures` origin. The
//! `ignis serve` binary injects it into the fixture origin only (never into the registry origin);
//! its releases are `trained_on = synthetic` and its artifacts are never better than `fit_only`.
//!
//! Method: ordinary least squares of d² on t over the admitted liquid-diameter prefix, predictive
//! Student-t(n-2) bands for d² mapped through sqrt (monotone, so quantiles map exactly).

use anyhow::bail;
use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, EvidenceClass, Family, JsonMap, ModelRelease, PrefixBundle,
    QualificationState, Quantity, Support, SupportGate,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData};
use ignis_core::hashing::content_hash;
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;
use serde_json::{json, Value};

pub const FIXTURE_NOTE: &str = "development fixture — synthetic";
pub const MODEL_NAME: &str = "fixture.synthetic_d2";
const DEFAULT_LEVELS: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];

/// The experts the `/fixtures` origin may use (synthetic only).
pub fn experts() -> Vec<Box<dyn ignis_core::expert::Expert>> {
    vec![Box::new(SyntheticD2Expert::new())]
}

pub struct SyntheticD2Expert {
    policy: ProspectivePolicy,
}

impl SyntheticD2Expert {
    pub fn new() -> Self {
        SyntheticD2Expert {
            policy: ProspectivePolicy::new(
                Family::Flex,
                &["pressure", "oxygen_mole_fraction"],
                &[],
            ),
        }
    }
}

impl Default for SyntheticD2Expert {
    fn default() -> Self {
        Self::new()
    }
}

/// Horner evaluation, coefficients from the highest degree down.
fn horner(c: &[f64], r: f64) -> f64 {
    c.iter().fold(0.0, |acc, &k| acc * r + k)
}

/// Standard normal quantile, Wichura AS241 (PPND16) — the algorithm of Python `NormalDist.inv_cdf`.
#[allow(clippy::excessive_precision)] // AS241 coefficients verbatim
pub fn norm_ppf(p: f64) -> f64 {
    let q = p - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180625 - q * q;
        let num = horner(
            &[
                2.509_080_928_730_122_672_7e3,
                3.343_057_558_358_812_810_5e4,
                6.726_577_092_700_870_085_3e4,
                4.592_195_393_154_987_145_7e4,
                1.373_169_376_550_946_112_5e4,
                1.971_590_950_306_551_442_7e3,
                1.331_416_678_917_843_774_5e2,
                3.387_132_872_796_366_608_0,
            ],
            r,
        ) * q;
        let den = horner(
            &[
                5.226_495_278_852_854_561_0e3,
                2.872_908_573_572_194_267_4e4,
                3.930_789_580_009_271_061_0e4,
                2.121_379_430_158_659_586_7e4,
                5.394_196_021_424_751_107_7e3,
                6.871_870_074_920_579_083_0e2,
                4.231_333_070_160_091_125_2e1,
                1.0,
            ],
            r,
        );
        return num / den;
    }
    let r = (-(if q <= 0.0 { p } else { 1.0 - p }).ln()).sqrt();
    let x = if r <= 5.0 {
        let r = r - 1.6;
        horner(
            &[
                7.745_450_142_783_414_076_40e-4,
                2.272_384_498_926_918_458_33e-2,
                2.417_807_251_774_506_117_70e-1,
                1.270_458_252_452_368_382_58,
                3.647_848_324_763_204_605_04,
                5.769_497_221_460_691_405_50,
                4.630_337_846_156_545_295_90,
                1.423_437_110_749_683_577_34,
            ],
            r,
        ) / horner(
            &[
                1.050_750_071_644_416_843_24e-9,
                5.475_938_084_995_344_946_00e-4,
                1.519_866_656_361_645_719_66e-2,
                1.481_039_764_274_800_745_90e-1,
                6.897_673_349_851_000_045_50e-1,
                1.676_384_830_183_803_849_40,
                2.053_191_626_637_758_821_87,
                1.0,
            ],
            r,
        )
    } else {
        let r = r - 5.0;
        horner(
            &[
                2.010_334_399_292_288_132_65e-7,
                2.711_555_568_743_487_578_15e-5,
                1.242_660_947_388_078_438_60e-3,
                2.653_218_952_657_612_309_30e-2,
                2.965_605_718_285_048_912_30e-1,
                1.784_826_539_917_291_335_80,
                5.463_784_911_164_114_369_90,
                6.657_904_643_501_103_777_20,
            ],
            r,
        ) / horner(
            &[
                2.044_263_103_389_939_785_64e-15,
                1.421_511_758_316_445_888_70e-7,
                1.846_318_317_510_054_681_80e-5,
                7.868_691_311_456_132_591_00e-4,
                1.487_536_129_085_061_485_25e-2,
                1.369_298_809_227_358_053_10e-1,
                5.998_322_065_558_879_376_90e-1,
                1.0,
            ],
            r,
        )
    };
    if q < 0.0 {
        -x
    } else {
        x
    }
}

/// Student-t quantile by the Cornish-Fisher expansion (adequate for a fixture, df >= 2).
pub fn t_quantile(p: f64, df: f64) -> f64 {
    let z = norm_ppf(p);
    let g1 = (z.powi(3) + z) / 4.0;
    let g2 = (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / 96.0;
    let g3 = (3.0 * z.powi(7) + 19.0 * z.powi(5) + 17.0 * z.powi(3) - 15.0 * z) / 384.0;
    let g4 = (79.0 * z.powi(9) + 776.0 * z.powi(7) + 1482.0 * z.powi(5)
        - 1920.0 * z.powi(3)
        - 945.0 * z)
        / 92160.0;
    z + g1 / df + g2 / df.powi(2) + g3 / df.powi(3) + g4 / df.powi(4)
}

fn diag(v: Value) -> JsonMap {
    match v {
        Value::Object(m) => m,
        _ => JsonMap::new(),
    }
}

impl SyntheticD2Expert {
    fn done(
        &self,
        mut a: AnalysisArtifact,
        status: QualificationState,
        support: Support,
        diagnostics: Value,
    ) -> AnalysisArtifact {
        a.status = status;
        a.support = support;
        a.diagnostics = diag(diagnostics);
        if a.status != QualificationState::FitOnly {
            a.quantile_levels.clear();
            a.quantiles.clear();
            a.times_s.clear();
        }
        a.id = content_hash(&a, &["id", "created_at"]);
        a
    }
}

fn refusal(gate: SupportGate, reason: impl Into<String>) -> Support {
    Support {
        admitted: false,
        gate,
        reasons: vec![reason.into()],
    }
}

impl Expert for SyntheticD2Expert {
    fn family(&self) -> Family {
        Family::Flex
    }
    fn model_name(&self) -> &str {
        MODEL_NAME
    }
    fn is_baseline(&self) -> bool {
        true
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"min_observations": 4}))
    }
    fn estimate_hours(
        &self,
        _config: &Value,
        _n_runs: usize,
        _n_obs: usize,
        _profile: &Profile,
    ) -> f64 {
        0.0
    }
    fn fit(
        &self,
        _data: &FitData<'_>,
        _config: &Value,
        _ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease> {
        bail!("the development fixture expert has no fitting procedure")
    }
    fn log_likelihood(&self, _release: &LoadedRelease, _view: &View) -> ExpertResult<f64> {
        bail!("the development fixture expert has no likelihood")
    }

    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        let rel = &release.release;
        let levels: Vec<f64> = match settings.get("quantile_levels").and_then(Value::as_array) {
            Some(a) if !a.is_empty() => a.iter().filter_map(Value::as_f64).collect(),
            _ => DEFAULT_LEVELS.to_vec(),
        };
        let n_points = settings
            .get("n_points")
            .and_then(Value::as_u64)
            .unwrap_or(60)
            .max(2) as usize;
        let min_obs = rel
            .config
            .get("min_observations")
            .and_then(Value::as_u64)
            .unwrap_or(4) as usize;
        let obs: Vec<(f64, f64)> = prefix
            .observations
            .iter()
            .filter(|o| {
                o.quantity == Quantity::LiquidDiameter
                    && !o.quality_flags.iter().any(|f| f == "flagged")
            })
            .filter_map(|o| o.value_si.map(|v| (o.time_s, v * v)))
            .collect();
        let body = AnalysisArtifact {
            id: String::new(),
            kind: ArtifactKind::PrefixForecast,
            model_release_id: rel.id.clone(),
            run_id: Some(prefix.run_id.clone()),
            time_origin: Some(prefix.time_origin.clone()),
            prefix_hash: prefix.input_hash.clone(),
            settings_hash: content_hash(settings, &[]),
            status: QualificationState::FitOnly,
            quantity: Some(Quantity::LiquidDiameter),
            unit: Some("m".into()),
            times_s: vec![],
            quantile_levels: levels.clone(),
            quantiles: vec![],
            mean: vec![],
            space_s_m: vec![],
            field_quantiles: vec![],
            field_mean: vec![],
            event_categories: vec![],
            event_cdf: vec![],
            support: Support {
                admitted: true,
                gate: SupportGate::Passed,
                reasons: vec![],
            },
            diagnostics: JsonMap::new(),
            evidence_class: EvidenceClass::ModelOutput,
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        };
        let n = obs.len();
        if n < min_obs.max(3) {
            return Ok(self.done(
                body,
                QualificationState::InsufficientInference,
                refusal(
                    SupportGate::Measurement,
                    format!("{n} admitted diameter observations; at least {min_obs} needed"),
                ),
                json!({"n_obs": n, "fixture": FIXTURE_NOTE}),
            ));
        }
        let nf = n as f64;
        let tbar = obs.iter().map(|o| o.0).sum::<f64>() / nf;
        let ybar = obs.iter().map(|o| o.1).sum::<f64>() / nf;
        let sxx: f64 = obs.iter().map(|o| (o.0 - tbar).powi(2)).sum();
        if sxx <= 0.0 || sxx.is_nan() {
            return Ok(self.done(
                body,
                QualificationState::InsufficientInference,
                refusal(
                    SupportGate::Measurement,
                    "admitted observations share one time; slope undefined",
                ),
                json!({"n_obs": n, "fixture": FIXTURE_NOTE}),
            ));
        }
        let slope = obs.iter().map(|o| (o.0 - tbar) * (o.1 - ybar)).sum::<f64>() / sxx;
        let icpt = ybar - slope * tbar;
        let s = (obs
            .iter()
            .map(|o| (o.1 - (icpt + slope * o.0)).powi(2))
            .sum::<f64>()
            / (nf - 2.0))
            .sqrt();
        if slope >= 0.0 {
            return Ok(self.done(
                body,
                QualificationState::Unsupported,
                refusal(
                    SupportGate::Semantic,
                    "prefix shows no d² decrease; the d²-law phase is not established",
                ),
                json!({"n_obs": n, "slope_m2_per_s": slope, "fixture": FIXTURE_NOTE}),
            ));
        }
        let t_end_median = -icpt / slope;
        let horizon = settings
            .get("horizon_s")
            .and_then(Value::as_f64)
            .unwrap_or((t_end_median - prefix.cutoff_s).max(0.0));
        let mut reasons = vec![format!("{FIXTURE_NOTE} expert, not a scientific model")];
        let stop = (prefix.cutoff_s + horizon).min(t_end_median);
        if stop <= prefix.cutoff_s {
            return Ok(self.done(
                body,
                QualificationState::Unsupported,
                refusal(
                    SupportGate::Horizon,
                    "median d² reaches zero before the cutoff",
                ),
                json!({"n_obs": n, "fixture": FIXTURE_NOTE}),
            ));
        }
        if stop < prefix.cutoff_s + horizon {
            reasons.push("horizon truncated where the median d² reaches zero".into());
        }
        let times: Vec<f64> = (0..n_points)
            .map(|i| prefix.cutoff_s + (stop - prefix.cutoff_s) * i as f64 / (n_points - 1) as f64)
            .collect();
        let df = nf - 2.0;
        let quantiles: Vec<Vec<f64>> = levels
            .iter()
            .map(|&q| {
                let tq = t_quantile(q, df);
                times
                    .iter()
                    .map(|&t| {
                        let scale = s * (1.0 + 1.0 / nf + (t - tbar).powi(2) / sxx).sqrt();
                        (icpt + slope * t + tq * scale).max(0.0).sqrt()
                    })
                    .collect()
            })
            .collect();
        let mut body = body;
        body.times_s = times;
        body.quantiles = quantiles;
        // Never better than fit_only, whatever the release claims.
        Ok(self.done(
            body,
            QualificationState::FitOnly,
            Support { admitted: true, gate: SupportGate::Passed, reasons },
            json!({"n_obs": n, "slope_m2_per_s": slope, "residual_sd_m2": s, "df": df,
                   "method": "OLS d² ~ t, Student-t predictive (Cornish-Fisher quantiles)", "fixture": FIXTURE_NOTE}),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppf_matches_python_normaldist() {
        // statistics.NormalDist().inv_cdf(p)
        assert!((norm_ppf(0.05) - -1.6448536269514726).abs() < 1e-14);
        assert!((norm_ppf(0.975) - 1.9599639845400536).abs() < 1e-14);
        assert!((norm_ppf(1e-10) - -6.361340902404056).abs() < 1e-12);
        assert_eq!(norm_ppf(0.5), 0.0);
    }
}
