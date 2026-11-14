# Runpod setup

Two small scripts for working on a Runpod GPU pod. **Neither one, nor anything else in this
repository, can create, stop or delete a pod.** Pods are started and stopped by a person in the Runpod
console, and billing state is confirmed there.

> [!WARNING]
> These scripts were written on Windows and have only been parsed (`bash -n`). They have **not** been
> run on a Linux pod: no pod was started to check them, and Linux build correctness cannot be verified
> here. Read the output of the first run on a real pod carefully.

## On the pod

Clone the repository onto the persistent volume (not the container disk), then:

```bash
tools/runpod/setup.sh
```

This installs the system build tools, Rust (`RUST_VERSION`, default `stable`) and `uv`, builds `ignis`
in release mode, installs `sidecar/sam2` and downloads the SAM2.1 checkpoint. Re-running is harmless.
Toolchains go to `/workspace/.cargo` and `/workspace/.rustup` when `/workspace` exists.

For a GPU run set `device = "cuda"` in `configs/vision/sam2.toml`, and dry-run first with the pod
profile: `ignis vision contours <clip> --calibration <spec> --method sam2 --profile runpod-3090`.

## From your machine

```bash
export SSH_OPTS="-p <port> -i ~/.ssh/<key>"
export REMOTE_DIR="<absolute-remote-checkout-path>"

# send only what the job needs
tools/runpod/sync.sh push root@<host> data/work/views/<cohort-hex> data/splits data/raw/samples/PSI-25/<clip>.mp4

# bring results back, then verify every file by sha256 on both sides
tools/runpod/sync.sh pull root@<host>
```

`pull` fetches `artifacts/{models,vision,analysis,edits}`, `runs/jobs` (the cost ledger) and
`runs/evaluation`, then compares `sha256sum` output from both sides. It exits non-zero and says not to
stop the pod on any mismatch; only after `verified: ... match by sha256` should the pod be stopped from
the console. `sync.sh` uses `rsync` when available and `scp` otherwise (Git Bash on Windows usually has
only `scp`).

Resource limits live in `configs/resources/runpod-3090.toml`: 16 vCPUs, 12 h per job, a 0.50 USD/h
planning tariff. Re-confirm the vCPU count and the tariff at checkout.
