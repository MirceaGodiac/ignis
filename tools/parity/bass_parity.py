"""HISTORICAL: needs the Python ACME/BASS reference deleted after the Rust port (last present at 56eb1d6).
The JSON fixtures it wrote are frozen under crates/ignis-{acme,bass}/tests/fixtures.

Freeze BASS parity fixtures from the Python/JAX reference (pre-deletion). Legacy numerics:
Lax-Friedrichs first order, linear ghosts. `uv run python tools/parity/bass_parity.py`."""
import json, time
import numpy as np
import jax, jax.numpy as jnp
jax.config.update("jax_enable_x64", True)
from ignis.contracts.views import BassSequence
from ignis.models.bass import data as D, dynamics as dyn, model as M, observation as O, operator as op
from ignis.models.bass.expert import load_config, _norm_dict, _runs, _one, _numerics

def seq(run_id, gid, seed, n_frames=6, occlude=3):
    rng = np.random.default_rng(seed)
    o2 = float(rng.uniform(0.18, 0.30)); flow = float(rng.uniform(0.05, 0.2))
    speed = 1.0e-3 * (1.0 + 2.0 * (o2 - 0.21)) * (1.0 + 0.5 * (flow - 0.1))
    W, L, S = 0.04, 0.1, 40
    s = np.linspace(0.0, W, S); xi = s / W
    f0 = 0.02 + 0.002 * np.cos(np.pi * xi) + 0.001 * rng.normal()
    t = np.arange(n_frames) * 1.0
    shape = 1.0 + 0.1 * np.cos(np.pi * xi)
    f = f0[None, :] + speed * t[:, None] * shape[None, :] + 5e-5 * rng.normal(size=(n_frames, S))
    valid = rng.uniform(size=f.shape) > 0.1
    status = ["single_valued"] * n_frames
    if occlude is not None:
        status[occlude] = "occluded"; valid[occlude] = False
    f = np.where(valid, f, np.nan)
    return BassSequence(run_id=run_id, group_id=gid, t_s=t, s_m=s, f_m=f, valid=valid, status=tuple(status),
                        conditions={"x_o2": o2, "pressure": 101325.0 * (1 + 0.1 * rng.normal()), "flow_velocity": flow,
                                    "thickness": 1.0e-4, "last_control_change_s": 0.0},
                        geometry={"width": W, "length": L}, scale_uncertainty_rel=np.full(n_frames, 0.005))

cfg = load_config("operator")
seqs = [seq("a", "g0", 1), seq("b", "g1", 2, occlude=None)]
prep = D.prepare(seqs, cfg["features"], cfg)
norm = D.fit_normalization(cfg["features"], prep)
nd = _norm_dict(cfg, norm)
runs = _runs(prep)
num = _numerics(cfg)
rng = np.random.default_rng(5)
g = op.init_network(jax.random.PRNGKey(0))
g = {k: v + 0.05 * jnp.asarray(rng.normal(size=v.shape)) for k, v in g.items()}
g |= {"physics.beta_v": jnp.asarray(rng.normal(0, 0.3, 9)).at[0].set(1.2), "physics.zeta": jnp.asarray(0.3)}
g |= {k: jnp.asarray(cfg["priors"][k]["mean"] + 0.2) for k in M.OBS_SHAPES}
x = jnp.stack([jnp.asarray(M.init_locals(_one(runs, i))) + 0.03 * jnp.asarray(rng.normal(size=12)) for i in range(2)])
r0 = _one(runs, 0)
cond = M.cond_fn_factory(r0["c_static"], r0["t_change"], nd["center"], nd["scale"], nd["is_time"], nd["ref"])
c = cond(r0["t"][1] + 0.3)
f0 = dyn.initial_front(x[0, :8], r0["L"], op.N_NODES)
chan = op.channels(f0, r0["L"], r0["W"])
out = op.forward(g, chan, c)
rate, alpha = dyn.rhs(g, g, x[0, 8:], f0, c, r0["L"], r0["W"])
F, miss, cfl = dyn.simulate(g, g, x[0, :8], x[0, 8:], r0["t"], cond, r0["L"], r0["W"], num)
sl = O.sequence_loglik(F, r0["xi"], r0["f"], r0["valid"], r0["use"], r0["u"], g, float(cfg["ell_s"]))
w = jnp.asarray([2.0, 1.0])
pr = cfg["priors"]
def obj(gg, xx):
    return M.member_objective(gg, xx, runs, w, nd, pr, num, float(cfg["ell_s"]))[0]
vg = jax.jit(jax.value_and_grad(obj, argnums=(0, 1)))
val, (gg, gx) = vg(g, x)
jax.block_until_ready(vg(g, x)); t0 = time.perf_counter(); n = 0
while time.perf_counter() - t0 < 3.0:
    jax.block_until_ready(vg(g, x)); n += 1
rate_vg = n / (time.perf_counter() - t0)
# gradient (sum of a random projection) through the rhs alone
proj = jnp.asarray(rng.normal(size=32))
rg = jax.grad(lambda gg, ff, bb: jnp.dot(proj, dyn.rhs(gg, gg, bb, ff, c, r0["L"], r0["W"])[0]), argnums=(0, 1, 2))(g, f0, x[0, 8:])
fx = {
    "sequences": [{"run_id": s.run_id, "group_id": s.group_id, "t_s": s.t_s.tolist(), "s_m": s.s_m.tolist(),
                   "f_m": [[None if not np.isfinite(v) else float(v) for v in row] for row in s.f_m],
                   "valid": s.valid.astype(bool).tolist(), "status": list(s.status), "conditions": s.conditions,
                   "geometry": s.geometry, "scale_uncertainty_rel": s.scale_uncertainty_rel.tolist()} for s in seqs],
    "normalization": norm,
    "params": {k: np.asarray(v).ravel().tolist() for k, v in g.items()}, "x": np.asarray(x).tolist(),
    "c": np.asarray(c).tolist(), "f0": np.asarray(f0).tolist(), "operator_out": np.asarray(out).tolist(),
    "rhs": np.asarray(rate).tolist(), "alpha": np.asarray(alpha).tolist(), "frames": np.asarray(F).tolist(),
    "unreached": float(miss), "max_cfl": float(cfl), "sequence_loglik": float(sl), "weights": [2.0, 1.0],
    "objective": float(val), "grad_g": {k: np.asarray(v).ravel().tolist() for k, v in gg.items()}, "grad_x": np.asarray(gx).tolist(),
    "rhs_proj": np.asarray(proj).tolist(), "rhs_grad_g": {k: np.asarray(v).ravel().tolist() for k, v in rg[0].items()},
    "rhs_grad_f": np.asarray(rg[1]).tolist(), "rhs_grad_b": np.asarray(rg[2]).tolist(),
    "jax_member_value_and_grad_per_s": rate_vg,
}
json.dump(fx, open("crates/ignis-bass/tests/fixtures/bass_parity.json", "w"))
print("jax member value+grad/s (2 runs):", rate_vg, "objective", float(val))
