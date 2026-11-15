//! Test double of the container engine CLI (`docker`) for the offline tests of `analysis_execute`
//! (M9). It answers the preflight queries, refuses a `run` without the isolation flags, and instead
//! of starting a container acts as one fixed program: it reports every file under the `/in` mount
//! (name, length, sha256) and the parsed `/job/inputs.json` on stdout, and writes the same report
//! to `/out/report.json`. The isolation probe gets a passing report. Container state lives in
//! `<temp>/ignis-mcp-fake-engine/<name>.json`. It never runs the submitted program: tests prove
//! what the host mounts and records, not what Python computes.

use std::path::{Path, PathBuf};

use ignis_core::hashing::sha256_hex;
use serde_json::{json, Value};

fn state_dir() -> PathBuf {
    let d = std::env::temp_dir().join("ignis-mcp-fake-engine");
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(125);
}

/// `source` of the bind mount whose target is `target`.
fn mount(args: &[String], target: &str) -> PathBuf {
    args.windows(2)
        .filter(|w| w[0] == "--mount")
        .find_map(|w| {
            let kv: Vec<(&str, &str)> = w[1].split(',').filter_map(|p| p.split_once('=')).collect();
            let get = |k: &str| kv.iter().find(|(a, _)| *a == k).map(|(_, v)| *v);
            (get("target") == Some(target)).then(|| PathBuf::from(get("source").unwrap()))
        })
        .unwrap_or_else(|| fail(&format!("no mount for {target}")))
}

fn report(inputs: &Path, job: &Path) -> Value {
    let mut files: Vec<Value> = std::fs::read_dir(inputs)
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            let bytes = std::fs::read(e.path()).unwrap();
            json!({
                "name": e.file_name().to_string_lossy(),
                "bytes": bytes.len(),
                "sha256": format!("sha256:{}", sha256_hex(&bytes)),
            })
        })
        .collect();
    files.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let manifest = std::fs::read(job.join("inputs.json"))
        .ok()
        .map(|b| serde_json::from_slice::<Value>(&b).unwrap());
    json!({"in": files, "inputs_json": manifest})
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["version", ..] => println!("linux"),
        ["info", ..] => println!(r#"{{"runc":{{}},"runsc":{{}}}}"#),
        ["image", "inspect", "--format", "{{.Id}}", image] => println!("{image}"),
        ["image", "inspect", "--format", _, _] => println!(r#"["PATH=/usr/local/bin:/usr/bin"]"#),
        ["ps", ..] | ["kill", ..] => {}
        ["rm", "-f", name] => {
            let _ = std::fs::remove_file(state_dir().join(format!("{name}.json")));
        }
        ["inspect", "--format", _, name] => {
            match std::fs::read_to_string(state_dir().join(format!("{name}.json"))) {
                Ok(s) => println!("{s}"),
                Err(_) => fail(&format!("Error: No such object: {name}")),
            }
        }
        ["run", ..] => run(&args),
        _ => fail(&format!("fake engine: unsupported {args:?}")),
    }
}

fn run(args: &[String]) {
    let has = |pair: [&str; 2]| args.windows(2).any(|w| w[0] == pair[0] && w[1] == pair[1]);
    if !(has(["--network", "none"]) && has(["--cap-drop", "ALL"]) && has(["--user", "65534:65534"]))
        || !args.iter().any(|x| x == "--read-only")
    {
        fail("fake engine: isolation flags missing");
    }
    let name = args
        .windows(2)
        .find(|w| w[0] == "--name")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| fail("no --name"));
    let (job, inputs, out) = (mount(args, "/job"), mount(args, "/in"), mount(args, "/out"));
    let program = std::fs::read_to_string(job.join("main.py")).unwrap_or_default();
    if program.contains("ignis_probe") {
        let input = std::fs::read_to_string(inputs.join("probe.txt")).unwrap_or_default();
        println!(
            "{}",
            json!({
                "uid": 65534, "gid": 65534, "cap_eff": "0000000000000000", "no_new_privs": "1",
                "root_writable": false, "in_writable": false, "out_writable": true,
                "input": input, "network": false, "ifaces": ["lo"], "docker_sock": false,
                "env": ["HOME", "PATH"], "python": "3.12.7",
            })
        );
    } else {
        let r = report(&inputs, &job);
        std::fs::write(out.join("report.json"), serde_json::to_vec(&r).unwrap()).unwrap();
        println!("{r}");
    }
    std::fs::write(
        state_dir().join(format!("{name}.json")),
        r#"{"ExitCode":0,"OOMKilled":false,"Error":""}"#,
    )
    .unwrap();
}
