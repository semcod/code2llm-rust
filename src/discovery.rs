//! Fast gitignore-aware parallel repository file discovery.

use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::Path;

/// Derive a module name from relative path, filename and project name (matching Python ProjectAnalyzer._compute_module_name).
pub fn compute_module_name(rel_path: &str, filename: &str, project_name: &str) -> String {
    let normalized = rel_path.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    let dir_parts = &parts[..parts.len().saturating_sub(1)];

    let is_init = matches!(
        filename,
        "__init__.py" | "index.js" | "index.ts" | "mod.rs" | "lib.rs"
    );

    if is_init {
        if dir_parts.is_empty() {
            project_name.to_string()
        } else {
            dir_parts.join(".")
        }
    } else {
        let stem = match filename.rfind('.') {
            Some(idx) => &filename[..idx],
            None => filename,
        };
        if dir_parts.is_empty() {
            stem.to_string()
        } else {
            format!("{}.{}", dir_parts.join("."), stem)
        }
    }
}

/// Check if a filename should be collected based on extension, exact name, or prefix.
pub fn should_collect_file(
    filename: &str,
    ext_set: &HashSet<String>,
    filename_set_lower: &HashSet<String>,
    filename_prefixes_lower: &[String],
) -> bool {
    let lower = filename.to_lowercase();
    let suffix = match lower.rfind('.') {
        Some(idx) => &lower[idx..],
        None => "",
    };

    if ext_set.contains(suffix) {
        return true;
    }
    if filename_set_lower.contains(&lower) {
        return true;
    }
    for prefix in filename_prefixes_lower {
        if lower.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Walk project directory collecting files matching criteria with early directory pruning.
pub fn walk_project_files(
    root: &str,
    extensions: &[String],
    filenames: &[String],
    filename_prefixes: &[String],
    skip_dirs: &[String],
    respect_gitignore: bool,
) -> Vec<(String, String)> {
    let root_path = Path::new(root);
    let project_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let ext_set: HashSet<String> = extensions.iter().map(|e| e.to_lowercase()).collect();
    let filename_set_lower: HashSet<String> = filenames.iter().map(|f| f.to_lowercase()).collect();
    let filename_prefixes_lower: Vec<String> =
        filename_prefixes.iter().map(|p| p.to_lowercase()).collect();
    let skip_dirs_set: HashSet<String> = skip_dirs.iter().map(|s| s.to_lowercase()).collect();

    let mut builder = WalkBuilder::new(root_path);
    builder
        .hidden(true)
        .git_ignore(respect_gitignore)
        .git_global(respect_gitignore)
        .git_exclude(respect_gitignore);

    let skip_dirs_clone = skip_dirs_set.clone();
    builder.filter_entry(move |entry| {
        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => return true,
        };
        if file_type.is_dir() {
            if let Some(name_os) = entry.path().file_name() {
                if let Some(name) = name_os.to_str() {
                    // Do not prune the root directory itself
                    if entry.depth() > 0 {
                        let lower = name.to_lowercase();
                        if lower.starts_with('.') || skip_dirs_clone.contains(&lower) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    });

    let mut results = Vec::new();
    let walker = builder.build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        if let Some(ft) = entry.file_type() {
            if !ft.is_file() {
                continue;
            }
        } else {
            continue;
        }

        let path = entry.path();
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(f) => f,
            None => continue,
        };

        if !should_collect_file(
            filename,
            &ext_set,
            &filename_set_lower,
            &filename_prefixes_lower,
        ) {
            continue;
        }

        let file_str = path.to_string_lossy().to_string();
        let rel_path = match path.strip_prefix(root_path) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let module_name = compute_module_name(&rel_path, filename, project_name);
        results.push((file_str, module_name));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_module_name() {
        assert_eq!(
            compute_module_name("foo/bar.py", "bar.py", "myproj"),
            "foo.bar"
        );
        assert_eq!(
            compute_module_name("foo/__init__.py", "__init__.py", "myproj"),
            "foo"
        );
        assert_eq!(
            compute_module_name("__init__.py", "__init__.py", "myproj"),
            "myproj"
        );
        assert_eq!(
            compute_module_name("src/utils/index.ts", "index.ts", "myproj"),
            "src.utils"
        );
        assert_eq!(
            compute_module_name("main.go", "main.go", "myproj"),
            "main"
        );
    }

    #[test]
    fn test_should_collect_file() {
        let ext_set: HashSet<String> = [".py".to_string(), ".ts".to_string()].into_iter().collect();
        let filenames: HashSet<String> = ["dockerfile".to_string(), "makefile".to_string()].into_iter().collect();
        let prefixes = vec!["dockerfile.".to_string(), "makefile.".to_string()];

        assert!(should_collect_file("foo.py", &ext_set, &filenames, &prefixes));
        assert!(should_collect_file("Dockerfile", &ext_set, &filenames, &prefixes));
        assert!(should_collect_file("Dockerfile.dev", &ext_set, &filenames, &prefixes));
        assert!(!should_collect_file("image.png", &ext_set, &filenames, &prefixes));
    }

    #[test]
    fn test_walk_project_files() {
        let root = env!("CARGO_MANIFEST_DIR");
        let ext = vec![".rs".to_string()];
        let files = walk_project_files(root, &ext, &[], &[], &["target".to_string()], true);
        assert!(!files.is_empty());
        assert!(files.iter().any(|(p, _)| p.ends_with("lib.rs")));
    }
}
