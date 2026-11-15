//! Outcome-free numerical check of the main author's frozen thermal block.
//!
//! Run: cargo run -p ignis-wsp --example astra_thermal_convergence -- --execute
//! Optionally supply --registry-root PATH --methanol-release ID --heptane-release ID.
//! Release-backed inputs must exactly match their handwritten addition.wsp. Without an ID,
//! the existing literal compiler reads that addition, and the explicit field mapping below
//! constructs the public Thermal value. It does not assemble or write any weights/releases.
//!
//! The main author specified both timesteps and every synthetic scenario in this checker.
//! This is one numerical comparison, not a step search, fitted model, error bound, or test of
//! empirical accuracy. No experimental observation, feedback, split or outcome is read.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use ignis_core::contracts::{ModelSpec, TensorFill};
use ignis_core::hashing::sha256_hex;
use ignis_core::registry::{self, Tensor, TensorMap};
use ignis_wsp::droplet7::{self, Fuel};
use ignis_wsp::thermal::{Gas, State, Thermal, Trajectories};
use ignis_wsp::{audit, text};
use serde_json::{json, Value};

#[derive(Parser)]
struct Args {
    /// Explicit opt-in to numerical execution; no network or experimental data are used.
    #[arg(long)]
    execute: bool,
    #[arg(long)]
    registry_root: Option<PathBuf>,
    #[arg(long)]
    methanol_release: Option<String>,
    #[arg(long)]
    heptane_release: Option<String>,
}

fn scalar(tensors: &TensorMap, suffix: &str) -> Result<f64> {
    Ok(array::<1>(tensors, suffix)?[0])
}

fn array<const N: usize>(tensors: &TensorMap, suffix: &str) -> Result<[f64; N]> {
    let name = format!("thermal.{suffix}");
    let t = tensors
        .get(&name)
        .with_context(|| format!("missing {name}"))?;
    ensure!(t.data.len() == N, "{name}: expected {N} values");
    ensure!(
        t.data.iter().all(|v| v.is_finite()),
        "{name}: non-finite value"
    );
    t.data
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("{name}: shape mismatch"))
}

fn binary(value: f64) -> Result<bool> {
    match value {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => bail!("binary thermal value must equal 0 or 1"),
    }
}

/// Explicit field binding is necessary because Thermal::bind is crate-private. All values
/// come from the public literal compiler or hash-checked registry tensors, never defaults.
fn bind(t: &TensorMap) -> Result<Thermal> {
    ensure!(
        scalar(t, "on")? == 1.0,
        "this check requires the enabled pure thermal block"
    );
    ensure!(
        scalar(t, "member_weight")? == 1.0,
        "supply the pure R1 release, not a joint member mixture"
    );
    let apply = array::<3>(t, "apply")?;
    let thermal = Thermal {
        nitrogen_only: binary(scalar(t, "nitrogen_only")?)?,
        cutoff_on: binary(scalar(t, "cutoff_on")?)?,
        apply: [binary(apply[0])?, binary(apply[1])?, binary(apply[2])?],
        dt_s: scalar(t, "dt_s")?,
        gas_constant_j_mol_k: scalar(t, "gas_constant_j_mol_k")?,
        molar_mass_kg_mol: scalar(t, "molar_mass_kg_mol")?,
        rho_kg_m3: scalar(t, "rho_kg_m3")?,
        cp_liquid_j_kg_k: scalar(t, "cp_liquid_j_kg_k")?,
        cp_gas_j_kg_k: scalar(t, "cp_gas_j_kg_k")?,
        k_gas_w_m_k: array(t, "k_gas_w_m_k")?,
        diffusivity_ref_m2_s: array(t, "diffusivity_ref_m2_s")?,
        transport_ref_k: scalar(t, "transport_ref_k")?,
        transport_ref_pa: scalar(t, "transport_ref_pa")?,
        diffusivity_exponent: scalar(t, "diffusivity_exponent")?,
        antoine: array(t, "antoine")?,
        antoine_range_k: array(t, "antoine_range_k")?,
        latent: array(t, "latent")?,
        latent_range_k: array(t, "latent_range_k")?,
        vapor_extension: binary(scalar(t, "vapor_extension")?)?,
        latent_extrapolation: binary(scalar(t, "latent_extrapolation")?)?,
        ambient_k: scalar(t, "ambient_k")?,
        cloud_excess_k: array(t, "cloud_excess_k")?,
        cloud_tau_s: array(t, "cloud_tau_s")?,
        initial_k: array(t, "initial_k")?,
        activity: array(t, "activity")?,
        x_infinity: scalar(t, "x_infinity")?,
        surface_fraction_max: scalar(t, "surface_fraction_max")?,
    };
    thermal.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        thermal.dt_s == 0.01,
        "authored dt changed: this frozen comparison requires 0.01 s"
    );
    ensure!(
        thermal.ambient_k == 300.0,
        "authored ambient changed: this scenario requires 300 K"
    );
    ensure!(
        thermal.apply == [true, false, false],
        "expected future hot-radiative applicability only"
    );
    ensure!(
        thermal.nitrogen_only && !thermal.cutoff_on,
        "expected nitrogen-only future-transition law"
    );
    ensure!(
        thermal.cloud_excess_k == [0.0; 3],
        "expected zero residual-cloud heating"
    );
    Ok(thermal)
}

/// Read literal assignments with the existing compiler and reject missing/duplicate elements.
/// This is literal consumption for evaluation, never computed candidate-weight generation.
fn literal_tensors(source: &str, spec: &ModelSpec) -> Result<TensorMap> {
    let operations = text::compile(source, spec)?;
    let mut cells: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();
    for op in operations {
        let name = op.tensor();
        ensure!(
            name.starts_with("thermal."),
            "addition contains non-thermal tensor {name}"
        );
        let tensor = spec
            .tensor(name)
            .with_context(|| format!("unknown tensor {name}"))?;
        let values = cells
            .entry(name.to_owned())
            .or_insert_with(|| vec![None; tensor.numel()]);
        for (index, value) in op.assignments(&tensor.shape).map_err(anyhow::Error::msg)? {
            ensure!(values[index].is_none(), "duplicate {name}[{index}]");
            values[index] = Some(value.get());
        }
    }
    let mut tensors = TensorMap::new();
    for tensor in spec
        .tensors
        .iter()
        .filter(|t| t.name.starts_with("thermal."))
    {
        let values = cells
            .remove(&tensor.name)
            .with_context(|| format!("missing literal {}", tensor.name))?;
        let data: Vec<f64> = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("missing literal {}[{i}]", tensor.name)))
            .collect::<Result<_>>()?;
        tensors.insert(
            tensor.name.clone(),
            Tensor::new(tensor.shape.iter().map(|&n| n as usize).collect(), data),
        );
    }
    ensure!(cells.is_empty(), "unconsumed literal tensors");
    Ok(tensors)
}

fn inputs(fuel: Fuel, id: Option<&str>, root: &Path) -> Result<(Thermal, f64, Value)> {
    let spec = droplet7::spec(fuel);
    let dir = match fuel {
        Fuel::Methanol => "methanol",
        Fuel::Heptane => "heptane",
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../configs/models/flex/wsp6/{dir}/r1-astra/addition.wsp"
    ));
    let source = std::fs::read_to_string(&path).with_context(|| path.display().to_string())?;
    let literals = literal_tensors(&source, &spec)?;
    let (tensors, mechanism) = if let Some(id) = id {
        let loaded = registry::load(root, id)?;
        spec.check_release(&loaded.release)?;
        ensure!(
            loaded
                .release
                .construction
                .as_ref()
                .map(|m| &m.forward_hash)
                == Some(&audit::forward_hash_of(fuel.model_name())),
            "release forward differs from this compiled forward"
        );
        for (name, literal) in &literals {
            ensure!(
                loaded.params.get(name) == Some(literal),
                "release {id}: {name} differs from frozen addition.wsp"
            );
        }
        (
            loaded.params,
            "hash-checked assembled registry params, exactly matched to literal addition",
        )
    } else {
        (literals, "literal addition.wsp compiled with public parser; explicit Thermal field binding; no assembled release claimed")
    };
    let thermal = bind(&tensors)?;
    let expected_initial = match fuel {
        Fuel::Methanol => 330.0,
        Fuel::Heptane => 360.0,
    };
    ensure!(
        thermal.initial_k[0] == expected_initial,
        "authored hot-liquid temperature changed"
    );
    let floor = spec
        .tensor("num.d_min_m")
        .context("missing fixed diameter floor")?;
    let d_floor = match &floor.fill {
        TensorFill::Fixed { values } if values.len() == 1 => values[0].get(),
        _ => bail!("diameter floor is not the expected fixed spec scalar"),
    };
    if id.is_some() {
        ensure!(
            tensors
                .get("num.d_min_m")
                .is_some_and(|t| t.data == [d_floor]),
            "release diameter floor differs from fixed spec"
        );
    }
    let provenance = json!({
        "model": fuel.model_name(), "spec_id": fuel.spec_id(),
        "forward_hash": audit::forward_hash_of(fuel.model_name()),
        "addition_path": path, "addition_sha256": sha256_hex(source.as_bytes()),
        "release_id": id, "loading": mechanism, "diameter_floor_m": d_floor,
        "authored_thermal_parameters": thermal,
    });
    Ok((thermal, d_floor * d_floor, provenance))
}

fn outcome(
    result: &std::result::Result<State, String>,
    paths: &Trajectories<'_>,
    q_floor: f64,
) -> Value {
    match result {
        Ok(state) => json!({
            "status": "ok", "state": state, "diameter_m": state.q_m2.sqrt(),
            "at_support_floor": state.q_m2 <= q_floor,
            "temperature_meaning": if state.q_m2 <= q_floor { "terminal floor state, not liquid temperature evolved to the requested horizon" } else { "liquid temperature at requested horizon" },
            "trace": paths.trace,
        }),
        Err(reason) => {
            json!({"status": "refused", "reason": reason, "trace_before_refusal": paths.trace})
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.execute {
        println!("Numerical comparison only; rerun with --execute. Optional --registry-root PATH --methanol-release ID --heptane-release ID uses verified assembled tensors. No weights, outcomes or release files are written.");
        return Ok(());
    }
    let root = args.registry_root.unwrap_or_else(registry::default_root);
    let mut cases = Vec::new();
    let mut sources = Vec::new();
    let mut failed_cases = 0usize;
    let mut max_diameter_difference_um: Option<f64> = None;
    let mut max_nonterminal_temperature_difference_k: Option<f64> = None;
    for (fuel, release_id) in [
        (Fuel::Methanol, args.methanol_release.as_deref()),
        (Fuel::Heptane, args.heptane_release.as_deref()),
    ] {
        let (authored, q_floor, provenance) = inputs(fuel, release_id, &root)?;
        sources.push(provenance);
        // 0.005 s is the main author's separately prescribed diagnostic step. It is not selected
        // by this program, scored, searched, or persisted into any candidate or release.
        let diagnostic = Thermal {
            dt_s: 0.005,
            ..authored.clone()
        };
        for diameter_m in [0.002_f64, 0.003] {
            for pressure_atm in [0.7, 1.0, 2.0] {
                // 101325 Pa/atm is an exact unit conversion, not a model coefficient.
                let gas = Gas {
                    pressure_pa: pressure_atm * 101_325.0,
                    x_co2: 0.0,
                    x_he: 0.0,
                };
                let q0 = diameter_m * diameter_m;
                let initial = authored.initial(q0, gas.pressure_pa, 0);
                let initial_report = match &initial {
                    Ok((state, capped)) => {
                        json!({"status": "ok", "state": state, "pressure_capped": capped})
                    }
                    Err(reason) => json!({"status": "refused", "reason": reason}),
                };
                let mut coarse = Trajectories::new(&authored, gas, q_floor);
                let mut fine = Trajectories::new(&diagnostic, gas, q_floor);
                for horizon_s in [5.0, 20.0, 45.0] {
                    let a = coarse.at(q0, horizon_s, 0);
                    let b = fine.at(q0, horizon_s, 0);
                    let differences = match (&a, &b) {
                        (Ok(a), Ok(b)) => {
                            let dd_um = (a.q_m2.sqrt() - b.q_m2.sqrt()).abs() * 1.0e6;
                            let dt_k = (a.temperature_k - b.temperature_k).abs();
                            let nonterminal = a.q_m2 > q_floor && b.q_m2 > q_floor;
                            max_diameter_difference_um =
                                Some(max_diameter_difference_um.map_or(dd_um, |m| m.max(dd_um)));
                            if nonterminal {
                                max_nonterminal_temperature_difference_k = Some(
                                    max_nonterminal_temperature_difference_k
                                        .map_or(dt_k, |m| m.max(dt_k)),
                                );
                            }
                            json!({"absolute_q_difference_m2": (a.q_m2-b.q_m2).abs(), "absolute_diameter_difference_um": dd_um,
                                "absolute_temperature_difference_k": dt_k, "both_temperatures_at_requested_horizon": nonterminal})
                        }
                        _ => {
                            failed_cases += 1;
                            Value::Null
                        }
                    };
                    cases.push(json!({"fuel": fuel.label(), "diameter_initial_m": diameter_m,
                        "pressure_atm": pressure_atm, "pressure_pa": gas.pressure_pa,
                        "horizon_since_hot_radiative_end_s": horizon_s, "initial": initial_report,
                        "authored_0_01_s": outcome(&a, &coarse, q_floor),
                        "diagnostic_0_005_s": outcome(&b, &fine, q_floor), "differences": differences}));
                }
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "purpose": "outcome-free numerical timestep comparison on fixed synthetic scenarios",
            "checker_source_sha256": sha256_hex(include_bytes!("astra_thermal_convergence.rs")),
            "compared_max_steps_s": [0.01, 0.005],
            "limits": "One prescribed halving comparison is neither an error bound nor empirical validation. No outcome, feedback, fitting, search or weight selection. Support-floor temperatures are terminal states, not predictions beyond the floor. Refusals remain visible and are never silently dropped.",
            "sources": sources, "cases": cases,
            "summary": {"requested_cases": cases.len(), "cases_with_any_refusal": failed_cases,
                "max_successful_diameter_difference_um": max_diameter_difference_um,
                "max_successful_nonterminal_temperature_difference_k": max_nonterminal_temperature_difference_k}
        }))?
    );
    Ok(())
}
