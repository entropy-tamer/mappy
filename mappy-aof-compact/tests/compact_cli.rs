use std::fs;

use mappy_aof_compact::{RunOptions, run};
use mappy_core::{Storage, StorageConfig};
use tempfile::TempDir;

#[tokio::test]
async fn e2e_compacted_aof_reloads_with_last_values() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().to_string_lossy().to_string();
    let aof_path = temp.path().join("mappy.aof");
    let body = concat!(
        "{\"Set\":{\"key\":\"live\",\"value\":[1,2]}}\n",
        "{\"Set\":{\"key\":\"dead\",\"value\":[3]}}\n",
        "{\"Delete\":{\"key\":\"dead\"}}\n",
        "{\"Set\":{\"key\":\"live\",\"value\":[4,5,6]}}\n"
    );
    fs::write(&aof_path, body).unwrap();

    let mut options = RunOptions::new(&aof_path);
    options.confirm_offline = true;
    let report = run(options).unwrap();
    assert_eq!(report.records_read, 4);
    assert_eq!(report.records_written, 1);
    let compacted = fs::read_to_string(&aof_path).unwrap();
    assert_eq!(
        compacted,
        "{\"Set\":{\"key\":\"live\",\"value\":[4,5,6]}}\n"
    );
    assert_eq!(
        fs::read_to_string(aof_path.with_extension("aof.pre-compact")).unwrap(),
        body
    );

    let storage = mappy_core::storage::aof::AOFStorage::new(StorageConfig {
        data_dir,
        sync_interval: 1,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(storage.get("live").await.unwrap(), Some(vec![4, 5, 6]));
    assert_eq!(storage.get("dead").await.unwrap(), None);
    drop(storage);
}

#[test]
fn analysis_reports_kept_bytes_without_changing_the_file() {
    let temp = TempDir::new().unwrap();
    let aof_path = temp.path().join("mappy.aof");
    let body = concat!(
        "{\"Set\":{\"key\":\"a\",\"value\":[1]}}\n",
        "{\"Set\":{\"key\":\"a\",\"value\":[2,3]}}\n"
    );
    fs::write(&aof_path, body).unwrap();

    let mut options = RunOptions::new(&aof_path);
    options.dry_run = true;
    let report = run(options).unwrap();
    assert_eq!(report.records_read, 2);
    assert_eq!(report.records_written, 1);
    assert_eq!(report.keys_retained, 1);
    assert_eq!(report.output_bytes, 34);
    assert_eq!(fs::read_to_string(&aof_path).unwrap(), body);
}

#[test]
fn compaction_refuses_without_explicit_offline_confirmation() {
    let temp = TempDir::new().unwrap();
    let aof_path = temp.path().join("mappy.aof");
    let body = "{\"Set\":{\"key\":\"live\",\"value\":[1]}}\n";
    fs::write(&aof_path, body).unwrap();

    let error = run(RunOptions::new(&aof_path)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("refusing to compact without --offline")
    );
    assert_eq!(fs::read_to_string(&aof_path).unwrap(), body);
}
