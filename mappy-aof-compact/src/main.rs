use std::env;

use anyhow::{Result, bail};
use mappy_aof_compact::{RunOptions, run};

fn main() -> Result<()> {
    let mut path = None;
    let mut skip_backup = false;
    let mut overwrite_backup = false;
    let mut confirm_offline = false;
    let mut dry_run = false;
    let args = env::args().skip(1);
    for arg in args {
        match arg.as_str() {
            "--skip-backup" => skip_backup = true,
            "--overwrite-backup" => overwrite_backup = true,
            "--offline" => confirm_offline = true,
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                println!(
                    "usage: mappy-aof-compact <AOF> [--offline | --dry-run] [--skip-backup] [--overwrite-backup]"
                );
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown option: {value}"),
            value => {
                if path.is_some() {
                    bail!("only one AOF path may be supplied");
                }
                path = Some(value.to_string());
            }
        }
    }
    let path = path.ok_or_else(|| {
        anyhow::anyhow!(
            "usage: mappy-aof-compact <AOF> [--offline | --dry-run] [--skip-backup] [--overwrite-backup]"
        )
    })?;
    let mut options = RunOptions::new(path);
    options.skip_backup = skip_backup;
    options.overwrite_backup = overwrite_backup;
    options.confirm_offline = confirm_offline;
    options.dry_run = dry_run;
    let report = run(options)?;
    println!(
        "input_bytes={} output_bytes={} records_read={} records_written={} keys_retained={} tombstones_dropped={}",
        report.input_bytes,
        report.output_bytes,
        report.records_read,
        report.records_written,
        report.keys_retained,
        report.tombstones_dropped
    );
    Ok(())
}
