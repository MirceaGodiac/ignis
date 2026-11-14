"""HISTORICAL: needs the Python ACME/BASS reference deleted after the Rust port (last present at 56eb1d6).
The JSON fixtures it wrote are frozen under crates/ignis-{acme,bass}/tests/fixtures.

Freeze ACME parity fixtures from the (pre-deletion) Python/JAX reference. Run once with the
Python still present: `uv run python tools/parity/acme_parity.py`. Output: crates/ignis-acme/tests/fixtures/acme_parity.json."""
import json, time
import numpy as np
import jax, jax.numpy as jnp
jax.config.update("jax_enable_x64", True)
from ignis.contracts.views import AcmeEventRow
from ignis.models.acme import model as M
from ignis.models.acme.expert import predict_partition
from ignis.science import gauss_hermite as gh

rng = np.random.default_rng(11)

def rows_fixture():
    rows = []
    for g in range(5):
        for i in range(6):
            entry = 1.0 + 0.5 * rng.uniform()
            kind = ["event", "interval", "right", "scheduled_stop", "interval", "event"][i]
            stop = 60.0 if i in (0, 3, 4) else None
            if kind == "event":
                t = entry + rng.uniform(1, 40)
                rows.append(AcmeEventRow(f"r{g}{i}", f"s{g}", "normal", {}, entry, kind, t, t,
                                         ("extinction",) if i == 0 else ("extinction", "operator_termination"), stop))
            elif kind == "interval":
                lo = entry + rng.uniform(0.5, 30); hi = lo + rng.uniform(0.5, 50)
                if stop is not None:
                    hi = 70.0  # includes the stop atom
                causes = ("extinction", "operator_termination") if i == 4 else ("extinction",)
                rows.append(AcmeEventRow(f"r{g}{i}", f"s{g}", "normal", {}, entry, kind, lo, hi, causes, stop))
            elif kind == "right":
                c = entry + rng.uniform(1, 90)
                rows.append(AcmeEventRow(f"r{g}{i}", f"s{g}", "normal", {}, entry, kind, c, None, (), stop))
            else:
                rows.append(AcmeEventRow(f"r{g}{i}", f"s{g}", "normal", {}, entry, kind, None, None, ("operator_termination",), stop))
    return rows

def row_json(r):
    return {"run_id": r.run_id, "group_id": r.group_id, "configuration": r.configuration, "features": {},
            "t_entry_s": r.t_entry_s, "censoring": r.censoring, "t_lower_s": r.t_lower_s, "t_upper_s": r.t_upper_s,
            "allowed_causes": list(r.allowed_causes), "scheduled_stop_s": r.scheduled_stop_s, "time_origin": "t0"}

out = {"cases": []}
rows = rows_fixture()
F = 8
C = rng.normal(0, 1, (len(rows), F))
for causes in (("extinction", "operator_termination"), ("extinction",)):
    for neural, linear, frailty in ((True, True, True), (False, True, False), (False, False, False)):
        spec = M.AcmeSpec(causes, (0.0, 5.0, 10.0, 20.0, 40.0, 80.0), 10.0, F, linear, neural, frailty, 8, 8)
        params = {}
        for name, shape in M.param_shapes(spec).items():
            params[name] = rng.normal(0, 0.4, shape)
        params["baseline.a"] = np.log(0.03 * 10) + rng.normal(0, 0.3, (spec.K, spec.n_bins))
        if frailty:
            params["frailty.log_sigma"] = np.log(rng.uniform(0.3, 0.7, spec.K))
        p = {k: jnp.asarray(v) for k, v in params.items()}
        enc = M.encode_rows(rows, C, spec)
        a = enc.arrays()
        z, logw = M.quadrature(spec, 12)
        z, logw = jnp.asarray(z), jnp.asarray(logw)
        pr = M.prior_settings(spec, {})
        G = enc.n_sessions
        f = jax.jit(jax.value_and_grad(lambda q: M.log_posterior(q, spec, pr, a, z, logw, G)))
        v, g = f(p)
        rl = M.row_loglik_nodes(p, spec, a, z)
        ml = M.marginal_loglik(p, spec, a, z, logw, G)
        lp = M.log_prior(p, spec, pr)
        # timing
        f(p)[0].block_until_ready()
        t0 = time.perf_counter(); n = 0
        while time.perf_counter() - t0 < 2.0:
            jax.block_until_ready(f(p)); n += 1
        rate = n / (time.perf_counter() - t0)
        case = {"spec": {"causes": list(causes), "bin_lower_edges_s": list(spec.bin_lower_edges_s), "t_ref_s": 10.0,
                         "n_features": F, "linear": linear, "neural": neural, "frailty": frailty, "hidden": 8, "cells_per_bin": 8},
                "params": {k: np.asarray(x).ravel().tolist() for k, x in params.items()},
                "row_loglik_nodes": np.asarray(rl).tolist(), "marginal_loglik": float(ml), "log_prior": float(lp),
                "log_posterior": float(v), "grad": {k: np.asarray(x).ravel().tolist() for k, x in g.items()},
                "jax_value_and_grad_per_s": rate}
        # forecast (two draws)
        if frailty:
            draws = {k: jnp.stack([jnp.asarray(x), jnp.asarray(x) + 0.2]) for k, x in params.items()}
            c = jnp.asarray(C[0])
            times = jnp.asarray([12.0, 20.0, 30.0, 45.0, 60.0, 75.0])
            fc = {}
            for nodes in (12, 24):
                zz, lw = M.quadrature(spec, nodes)
                o = predict_partition(draws, spec, c, 2.0, 10.0, times, 60.0, 80.0, jnp.asarray(zz), jnp.asarray(lw))
                fc[str(nodes)] = {k: np.asarray(o[k]).tolist() for k in ("active", "F", "atom", "unresolved", "global")}
            # known session table (normalised per draw by the reviewed code)
            tab = jax.vmap(lambda q: M.session_loglik_nodes(q, spec, a, z, G))(draws)
            o = predict_partition(draws, spec, c, 2.0, 10.0, times, 60.0, 80.0, z, logw, tab[:, 1, :])
            fc["known_session_1"] = {k: np.asarray(o[k]).tolist() for k in ("active", "F", "atom", "unresolved", "global")}
            case["forecast"] = {"c": C[0].tolist(), "t_entry": 2.0, "tau": 10.0, "times": times.tolist(), "stop": 60.0,
                                "horizon": 80.0, "draw_shift": 0.2, "results": fc,
                                "session_table_draw0": np.asarray(tab[0]).tolist()}
        out["cases"].append(case)
out["rows"] = [row_json(r) for r in rows]
out["c"] = C.tolist()
out["gh"] = {str(n): [x.tolist() for x in gh.hermite_rule(n)] for n in (2, 5, 12, 24, 48)}
json.dump(out, open("crates/ignis-acme/tests/fixtures/acme_parity.json", "w"))
print("cases", len(out["cases"]), [c["jax_value_and_grad_per_s"] for c in out["cases"]])
