//! Bounded Rayon pool for CPU batch scoring. Created once by the server with an explicit thread
//! count so Rayon, BLAS and the JAX worker do not oversubscribe the host (§14.2).

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::kernels::{student_t_ou_loglik, OuParams};

pub fn bounded_pool(threads: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .thread_name(|i| format!("ignis-score-{i}"))
        .build()
        .expect("rayon pool")
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResidualSeries {
    pub times_s: Vec<f64>,
    pub residuals: Vec<f64>,
    pub sigma_c: f64,
    pub ell_s: f64,
    pub sigma_n: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SeriesScore {
    Ok { loglik: f64, n: usize },
    Rejected { reason: String },
}

pub fn score_batch(pool: &rayon::ThreadPool, series: &[ResidualSeries]) -> Vec<SeriesScore> {
    pool.install(|| {
        series
            .par_iter()
            .map(|s| {
                let p = OuParams {
                    sigma_c: s.sigma_c,
                    ell: s.ell_s,
                    sigma_n: s.sigma_n,
                };
                match student_t_ou_loglik(&s.times_s, &s.residuals, p) {
                    Ok(loglik) => SeriesScore::Ok {
                        loglik,
                        n: s.times_s.len(),
                    },
                    Err(e) => SeriesScore::Rejected {
                        reason: e.to_string(),
                    },
                }
            })
            .collect()
    })
}
