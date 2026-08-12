use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// GUI-launched Linux processes (desktop entry, AppImage) frequently see a
/// `PATH` that excludes directories a login shell picks up from
/// `.bashrc`/`.profile`/nvm/pnpm init scripts — the official Tauri Linux
/// distribution docs call this out explicitly. Dependency discovery
/// (`installer::dependency`) therefore never trusts `PATH` alone: it probes
/// a fixed list of common Node/npm install locations and appends whichever
/// of them actually exist, preserving the original `PATH` order first so an
/// explicit user `PATH` entry always wins.
pub fn augmented_search_paths() -> Vec<PathBuf> {
    let path_var = env::var_os("PATH");
    let home = env::var("HOME").ok();
    compute_augmented_paths(path_var.as_deref(), home.as_deref())
}

fn compute_augmented_paths(path_var: Option<&OsStr>, home: Option<&str>) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = path_var
        .map(|v| env::split_paths(v).collect())
        .unwrap_or_default();

    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ];

    if let Some(home) = home {
        let home = Path::new(home);
        candidates.push(home.join(".local/share/pnpm"));
        candidates.push(home.join(".npm-global/bin"));
        candidates.push(home.join(".local/bin"));
        candidates.push(home.join(".cargo/bin"));

        // nvm installs each Node version under its own versioned bin dir
        // rather than one stable path, so this has to enumerate them.
        if let Ok(entries) = std::fs::read_dir(home.join(".nvm/versions/node")) {
            for entry in entries.flatten() {
                candidates.push(entry.path().join("bin"));
            }
        }
    }

    for candidate in candidates {
        if candidate.is_dir() && !paths.contains(&candidate) {
            paths.push(candidate);
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn preserves_existing_path_order_first() {
        let tmp = tempfile::tempdir().unwrap();
        let bin_a = tmp.path().join("bin_a");
        fs::create_dir_all(&bin_a).unwrap();
        let path_var = env::join_paths([&bin_a]).unwrap();

        let result = compute_augmented_paths(Some(path_var.as_os_str()), None);
        assert_eq!(result.first(), Some(&bin_a));
    }

    #[test]
    fn appends_existing_home_relative_candidates() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_bin = tmp.path().join(".cargo/bin");
        fs::create_dir_all(&cargo_bin).unwrap();

        let result = compute_augmented_paths(None, Some(tmp.path().to_str().unwrap()));
        assert!(result.contains(&cargo_bin));
    }

    #[test]
    fn discovers_nvm_versioned_bin_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let node_bin = tmp.path().join(".nvm/versions/node/v20.11.0/bin");
        fs::create_dir_all(&node_bin).unwrap();

        let result = compute_augmented_paths(None, Some(tmp.path().to_str().unwrap()));
        assert!(result.contains(&node_bin));
    }

    #[test]
    fn skips_candidates_that_do_not_exist() {
        let tmp = tempfile::tempdir().unwrap();
        // No .cargo/bin created under this home.
        let result = compute_augmented_paths(None, Some(tmp.path().to_str().unwrap()));
        assert!(!result.contains(&tmp.path().join(".cargo/bin")));
    }

    #[test]
    fn does_not_duplicate_paths_already_present() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_bin = tmp.path().join(".cargo/bin");
        fs::create_dir_all(&cargo_bin).unwrap();
        let path_var = env::join_paths([&cargo_bin]).unwrap();

        let result = compute_augmented_paths(
            Some(path_var.as_os_str()),
            Some(tmp.path().to_str().unwrap()),
        );
        assert_eq!(result.iter().filter(|p| *p == &cargo_bin).count(), 1);
    }
}
