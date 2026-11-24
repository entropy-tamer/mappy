//! Offline compaction for mappy append-only files.
//!
//! The compactor is deliberately single-purpose: it rewrites an AOF with one
//! final `Set` per live key, keeps a hard-link backup, and refuses to run while
//! another process still holds the AOF open.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mappy_core::{AofCompactionReport, CompactOptions, analyze_aof, compact_aof};

/// Options for one compaction run.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Path to the AOF to compact.
    pub aof: PathBuf,
    /// Skip the hard-link backup. Use only when a backup already exists.
    pub skip_backup: bool,
    /// Replace an existing hard-link backup.
    pub overwrite_backup: bool,
    /// Maximum number of open descriptors to report when refusing a run.
    pub max_open_holders: usize,
    /// Explicit confirmation that the owning service is stopped.
    pub confirm_offline: bool,
    /// Analyze only; leave the AOF untouched.
    pub dry_run: bool,
}

impl RunOptions {
    /// Build options for an AOF path with conservative defaults.
    pub fn new(aof: impl Into<PathBuf>) -> Self {
        Self {
            aof: aof.into(),
            skip_backup: false,
            overwrite_backup: false,
            max_open_holders: 5,
            confirm_offline: false,
            dry_run: false,
        }
    }
}

/// Run one compaction and return its metrics.
pub fn run(options: RunOptions) -> Result<AofCompactionReport> {
    let aof = canonical_existing(&options.aof)?;
    if options.dry_run {
        return analyze_aof(&aof).with_context(|| format!("failed to analyze {}", aof.display()));
    }
    if !options.confirm_offline {
        bail!("refusing to compact without --offline; stop the owning service first");
    }
    let holders = open_holders(&aof)?;
    if !holders.is_empty() {
        let shown = holders
            .iter()
            .take(options.max_open_holders)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "refusing to compact while {} process(es) hold {}: {}",
            holders.len(),
            aof.display(),
            shown
        );
    }

    let compact_options = CompactOptions {
        backup_path: None,
        overwrite_backup: options.overwrite_backup,
        skip_backup: options.skip_backup,
    };
    compact_aof(&aof, compact_options)
        .with_context(|| format!("failed to compact {}", aof.display()))
}

fn canonical_existing(path: &Path) -> Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("cannot resolve {}", path.display()))?;
    if !canonical.is_file() {
        bail!("{} is not a file", canonical.display());
    }
    Ok(canonical)
}

fn open_holders(aof: &Path) -> Result<Vec<String>> {
    let proc = Path::new("/proc");
    if !proc.is_dir() {
        return Ok(Vec::new());
    }
    let mut holders = Vec::new();
    let entries = std::fs::read_dir(proc).context("cannot enumerate /proc")?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let fd_dir = entry.path().join("fd");
        let fds = match std::fs::read_dir(fd_dir) {
            Ok(fds) => fds,
            Err(_) => continue,
        };
        for fd in fds.flatten() {
            let target = match std::fs::read_link(fd.path()) {
                Ok(target) => target,
                Err(_) => continue,
            };
            if let Ok(canonical) = std::fs::canonicalize(&target)
                && canonical == aof
            {
                holders.push(format!("pid {}", name));
                break;
            }
        }
    }
    holders.sort();
    holders.dedup();
    Ok(holders)
}
