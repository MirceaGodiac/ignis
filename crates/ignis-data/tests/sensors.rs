//! R2b: sensor text → `DatasetBundle` on synthetic files (`synthetic_fixture` content in the shape
//! of the PSI exports; no NASA bytes): build, store, serve through `DatasetService`, read the
//! Parquet back and follow every locator to its original cell.

use std::collections::HashMap;
use std::path::Path;

use arrow_array::{Array, Float64Array, RecordBatch, StringArray};
use ignis_core::contracts::{
    Capability, DatasetId, ExecutionScope, Partition, PrincipalId, ScopeId, ScopeRole, WorkspaceId,
};
use ignis_core::services::DatasetService;
use ignis_data::dataset::DatasetStore;
use ignis_data::sensors::{self, Format};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

const MDCA: &str = "\"FCU TimeStamp\",\"FCU_MP_PT16(PSIA)\",\"FCU_MP_TM1(Deg C)\",\"MAIN_COMPUTER_TIME(GMT)\",\"FAN_STATE(N/A)\",\"ODD(furlongs)\",\"COUNT(N/A)\"
\"2016-04-18 15:00:17.500000\",14.70,23.00,2016-04-18 15:00:17,On,1.5,3
\"2016-04-18 15:00:18.500000\",14.71,,2016-04-18 15:00:18,On,1.6,4
\"2016-04-18 15:00:19.500000\",14.72,23.10,2016-04-18 15:00:19,Off,1.7,5
";

const SHEET: &str = "Test Point:\t\t\t\tA001D01
AS Burn to Completion/Igniter On:\t2016-04-18 15:00:18.000
";

// Crosses midnight; the third and fourth rows share a timestamp and differ in `TCA` only.
const ACME: &str = "2019-04-25 115 23:59:58:000 ,Experiment: 0,Test Point: 17319A3,MFCA1 Gas: 0
Time,TCA,PMT1
23:59:59:900,0.5,4.5
23:59:59:950,0.6,4.6
00:00:00:050,0.7,4.7
00:00:00:050,0.8,4.7
";

const TP: &str =
    "\n\nTest Point: 17319A3\n\nIgnition and retraction: 2019-04-25 23:59:59\nMFCs off: ***\n";

fn write(dir: &Path, name: &str, text: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn scope(partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("test").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role: ScopeRole::Research,
        partitions: partitions.iter().copied().collect(),
        capabilities: [Capability::ReadData].into(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn read(store: &DatasetStore, sc: &ExecutionScope, id: &str, view: &str) -> RecordBatch {
    let p = store
        .view_path(sc, &DatasetId::new(id).unwrap(), view)
        .unwrap();
    let r = ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(p).unwrap())
        .unwrap()
        .with_batch_size(1 << 20)
        .build()
        .unwrap();
    let mut batches: Vec<RecordBatch> = r.map(|b| b.unwrap()).collect();
    assert_eq!(batches.len(), 1);
    batches.remove(0)
}

fn s<'a>(b: &'a RecordBatch, c: &str) -> &'a StringArray {
    b.column_by_name(c)
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap()
}
fn f<'a>(b: &'a RecordBatch, c: &str) -> &'a Float64Array {
    b.column_by_name(c)
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
}

#[test]
fn sensor_text_becomes_a_bundle_whose_values_trace_to_their_cells() {
    let tmp = tempfile::tempdir().unwrap();
    let raw = tmp.path().join("raw");
    write(&raw.join("PSI-68"), "A001D01_Chamber_Readings.txt", MDCA);
    write(&raw.join("PSI-68"), "A001D01.txt", SHEET);
    write(&raw.join("PSI-68"), "A001D01_Comments.txt", "crew note\n");
    assert_eq!(sensors::detect(MDCA), Some(Format::Mdca));
    assert_eq!(sensors::detect(ACME), Some(Format::AcmeLog));
    assert_eq!(sensors::detect(SHEET), None);

    let store = DatasetStore {
        builder_root: tmp.path().join("datasets"),
        evaluator_root: tmp.path().join("holdout"),
    };
    let b = sensors::build_investigation("PSI-68", &raw.join("PSI-68"), &store).unwrap();
    assert_eq!(b.files.len(), 1);
    assert_eq!(
        b.skipped.len(),
        2,
        "the sheet and the comments are not sensor tables"
    );
    assert_eq!(b.bundle.dataset_id.as_str(), "sensors-psi-68");
    assert_eq!(b.bundle.time_origin, "first_sample_of_file");

    // Served like any builder bundle; a scope without `train` does not get it.
    let sc = scope(&[Partition::Train, Partition::Validation]);
    let served = store
        .dataset(&sc, &DatasetId::new("sensors-psi-68").unwrap())
        .unwrap();
    assert_eq!(served, b.bundle);
    assert!(store
        .dataset(
            &scope(&[Partition::Validation]),
            &DatasetId::new("sensors-psi-68").unwrap()
        )
        .is_err());

    let t = read(&store, &sc, "sensors-psi-68", "sensor_samples");
    assert_eq!(t.num_rows(), 3 * 6);
    let text = std::fs::read_to_string(raw.join("PSI-68/A001D01_Chamber_Readings.txt")).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let mut by_status: HashMap<String, usize> = HashMap::new();
    for i in 0..t.num_rows() {
        *by_status
            .entry(s(&t, "status").value(i).to_string())
            .or_default() += 1;
        // The locator names the file by its bytes' hash and the original cell.
        let loc = s(&t, "locator").value(i);
        let (sid, cell) = loc.split_once("#line=").unwrap();
        assert_eq!(sid, b.files[0].source_id);
        let (line, col) = cell.split_once("&column=").unwrap();
        let fields: Vec<&str> = lines[line.parse::<usize>().unwrap() - 1]
            .split(',')
            .collect();
        assert_eq!(
            fields[col.parse::<usize>().unwrap() - 1].trim_matches('"'),
            s(&t, "raw_value").value(i)
        );
        assert_eq!(s(&t, "run_id").value(i), "PSI-68:A001D01");
        assert_eq!(s(&t, "stream").value(i), "Chamber_Readings");
    }
    let n = |k: &str| by_status.get(k).copied().unwrap_or(0);
    assert_eq!(n("ok"), 5, "PSIA ×3, Deg C ×2");
    assert_eq!(
        n("missing"),
        1,
        "the empty temperature cell is missing, not zero"
    );
    assert_eq!(
        n("text"),
        6,
        "GMT timestamps and On/Off words are kept as text"
    );
    assert_eq!(n("unit_not_converted"), 3);
    assert_eq!(n("unit_not_applicable"), 3);
    for i in 0..t.num_rows() {
        let ch = s(&t, "channel").value(i);
        if ch == "FCU_MP_PT16" && f(&t, "time_s").value(i) == 0.0 {
            assert_eq!(s(&t, "raw_unit").value(i), "PSIA");
            assert_eq!(f(&t, "raw_number").value(i), 14.70);
            assert!((f(&t, "value_si").value(i) - 14.70 * 6894.757293168).abs() < 1e-6);
            assert_eq!(s(&t, "unit_si").value(i), "Pa");
        }
        if ch == "FCU_MP_TM1" && s(&t, "status").value(i) == "ok" {
            assert!(
                (f(&t, "value_si").value(i) - (f(&t, "raw_number").value(i) + 273.15)).abs() < 1e-9
            );
            assert_eq!(s(&t, "unit_si").value(i), "K");
        }
        if s(&t, "status").value(i) != "ok" {
            assert!(
                f(&t, "value_si").is_null(i),
                "no SI value without a converted unit"
            );
        }
    }
    let times: Vec<f64> = (0..t.num_rows())
        .map(|i| f(&t, "time_s").value(i))
        .collect();
    assert!(times.contains(&0.0) && times.contains(&2.0));

    let files = read(&store, &sc, "sensors-psi-68", "sensor_files");
    assert_eq!(files.num_rows(), 1);
    assert_eq!(
        f(&files, "ignition_s").value(0),
        0.5,
        "igniter on 0.5 s after the first sample"
    );
    assert!(s(&files, "ignition_locator").value(0).ends_with("#line=2"));
}

#[test]
fn acme_logs_cross_midnight_and_mark_conflicting_repeats() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("PSI-21");
    write(&dir, "17319A3_data.csv", ACME);
    write(&dir, "17319A3_TP.txt", TP);
    let store = DatasetStore {
        builder_root: tmp.path().join("datasets"),
        evaluator_root: tmp.path().join("holdout"),
    };
    let b = sensors::build_investigation("PSI-21", &dir, &store).unwrap();
    let sc = scope(&[Partition::Train]);
    let t = read(&store, &sc, "sensors-psi-21", "sensor_samples");
    assert_eq!(t.num_rows(), 4 * 2);
    let mut seen = Vec::new();
    for i in 0..t.num_rows() {
        let (ch, st, ts) = (
            s(&t, "channel").value(i),
            s(&t, "status").value(i),
            f(&t, "time_s").value(i),
        );
        seen.push((ch.to_string(), st.to_string(), (ts * 1000.0).round() as i64));
        assert!(s(&t, "raw_unit").is_null(i), "rack logs state no unit");
        assert!(f(&t, "value_si").is_null(i));
    }
    // 23:59:59.900 → 00:00:00.050 the next day = 0.150 s, not −86399.85 s.
    assert!(seen.contains(&("TCA".into(), "conflicting".into(), 150)));
    assert_eq!(
        seen.iter().filter(|x| x.1 == "conflicting").count(),
        2,
        "both TCA rows"
    );
    assert_eq!(
        seen.iter()
            .filter(|x| x.0 == "PMT1" && x.1 == "unit_not_stated")
            .count(),
        4,
        "PMT1 agrees in the repeated row"
    );
    let files = read(&store, &sc, "sensors-psi-21", "sensor_files");
    assert_eq!(
        s(&files, "first_sample_utc").value(0),
        "2019-04-25T23:59:59.900000"
    );
    assert!(
        (f(&files, "ignition_s").value(0) - (-0.9)).abs() < 1e-9,
        "ignition 0.9 s before the first sample"
    );
    assert_eq!(b.quality["status_counts"]["conflicting"], 2);
}

#[test]
fn lockbox_and_unknown_investigations_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let store = DatasetStore {
        builder_root: tmp.path().join("datasets"),
        evaluator_root: tmp.path().join("holdout"),
    };
    for inv in ["PSI-69", "PSI-159", "PSI-25", "PSI-26"] {
        let e = sensors::build_investigation(inv, tmp.path(), &store)
            .err()
            .unwrap();
        assert!(e.to_string().contains("lockbox"), "{inv}: {e}");
    }
    assert!(sensors::build_investigation("PSI-999", tmp.path(), &store).is_err());
}

#[test]
fn file_names_give_test_point_and_stream() {
    assert_eq!(
        sensors::test_point("PSI-10_Raw Data_19115A1_data.csv"),
        "19115A1"
    );
    assert_eq!(
        sensors::stream_of("PSI-10_Raw Data_19115A1_data.csv", "19115A1"),
        "data"
    );
    assert_eq!(
        sensors::test_point("A109D03_Chamber_Readings.txt"),
        "A109D03"
    );
    assert_eq!(
        sensors::stream_of("A109D03_Chamber_Readings.txt", "A109D03"),
        "Chamber_Readings"
    );
}
