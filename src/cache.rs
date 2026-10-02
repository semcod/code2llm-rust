//! Fast parallel content hashing and cache validation for code2llm.

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Compute 16-hex-char content hash including analyzer version and filepath.
/// Matches Python: hashlib.sha256(f"v{analyzer_version}\x00{filepath}\x00".encode() + data).hexdigest()[:16]
pub fn compute_file_content_hash(filepath: &str, analyzer_version: &str) -> Option<String> {
    let data = fs::read(filepath).ok()?;
    let mut hasher = Sha256::new();
    let prefix = format!("v{}\0{}\0", analyzer_version, filepath);
    hasher.update(prefix.as_bytes());
    hasher.update(&data);
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    use std::fmt::Write;
    for b in result {
        let _ = write!(hex, "{:02x}", b);
    }
    Some(hex[..16.min(hex.len())].to_string())
}

/// Result of checking changed files.
pub struct CacheCheckResult {
    pub changed: Vec<String>,
    pub cached: Vec<String>,
    pub refreshed: Vec<(String, f64)>, // (filepath, new_mtime)
}

/// Check filepaths against manifest entries in parallel using Rayon.
/// manifest_entries: rel_path -> (hash, mtime, size)
pub fn check_changed_files_parallel(
    project_dir: &str,
    filepaths: &[String],
    manifest_entries: &HashMap<String, (String, f64, u64)>,
    analyzer_version: &str,
) -> CacheCheckResult {
    let proj_path = Path::new(project_dir);

    enum FileStatus {
        Changed(String),
        Cached(String),
        CachedRefreshed(String, f64),
    }

    let results: Vec<FileStatus> = filepaths
        .par_iter()
        .map(|fp| {
            let path = Path::new(fp);
            let rel = match path.strip_prefix(proj_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => return FileStatus::Changed(fp.clone()),
            };

            let prev = match manifest_entries.get(&rel) {
                Some(entry) => entry,
                None => return FileStatus::Changed(fp.clone()),
            };

            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => return FileStatus::Changed(fp.clone()),
            };

            let mtime_sec = match metadata.modified() {
                Ok(t) => match t.duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_secs_f64(),
                    Err(_) => 0.0,
                },
                Err(_) => 0.0,
            };
            let size = metadata.len();

            // L1: mtime and size match
            if (mtime_sec - prev.1).abs() < 1e-6 && size == prev.2 {
                return FileStatus::Cached(fp.clone());
            }

            // L2: content hash check (size may match but mtime drifted)
            if let Some(h) = compute_file_content_hash(fp, analyzer_version) {
                if h == prev.0 {
                    return FileStatus::CachedRefreshed(fp.clone(), mtime_sec);
                }
            }

            FileStatus::Changed(fp.clone())
        })
        .collect();

    let mut changed = Vec::new();
    let mut cached = Vec::new();
    let mut refreshed = Vec::new();

    for res in results {
        match res {
            FileStatus::Changed(fp) => changed.push(fp),
            FileStatus::Cached(fp) => cached.push(fp),
            FileStatus::CachedRefreshed(fp, new_mtime) => {
                cached.push(fp.clone());
                refreshed.push((fp, new_mtime));
            }
        }
    }

    CacheCheckResult {
        changed,
        cached,
        refreshed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_file_content_hash() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let test_file = manifest_dir.join("Cargo.toml");
        let h = compute_file_content_hash(test_file.to_str().unwrap(), "0.1.0").unwrap();
        assert_eq!(h.len(), 16);
    }
}
