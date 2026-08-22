use std::env;
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// GUI-launched Linux processes (desktop entry, AppImage) frequently see a
/// `PATH` that excludes directories a login shell picks up from
/// `.bashrc`/`.profile`/nvm/pnpm init scripts — the official Tauri Linux
/// distribution docs call this out explicitly. Dependency discovery
/// (`installer::dependency`) therefore never trusts `PATH` alone: it probes
/// a fixed list of common Node/npm install locations, appends whichever of
/// them actually exist, and — since any fixed list is inevitably incomplete
/// for less common setups (asdf, mise, or a bespoke tool directory a user's
/// shell rc exports directly) — also asks `$SHELL` itself for its resolved
/// `PATH`, exactly like the "fix-path-env" trick VS Code and others use for
/// the same GUI-launch problem. The original `PATH` order is preserved
/// first, so an explicit user `PATH` entry always wins.
pub fn augmented_search_paths() -> Vec<PathBuf> {
    let path_var = env::var_os("PATH");
    let home = env::var("HOME").ok();
    let mut paths = compute_augmented_paths(path_var.as_deref(), home.as_deref());
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    for dir in login_shell_path_dirs(&shell, Duration::from_secs(3)) {
        if !paths.contains(&dir) {
            paths.push(dir);
        }
    }
    paths
}

/// Runs `$SHELL -ilc 'echo -n "$PATH"'` and parses the result. `-i`
/// (interactive) is included alongside `-l` (login) because PATH exports
/// commonly live in interactive-only rc files (`.bashrc`/`.zshrc`) rather
/// than login ones (`.profile`/`.zprofile`), and shells only source those
/// when actually running interactively. Bounded by `timeout` and run
/// without a controlling tty (`Stdio::null()` for stdin) so a hung or
/// prompting rc script can never block dependency discovery indefinitely —
/// on timeout, or on any spawn/parse failure, this simply contributes no
/// extra paths rather than erroring.
fn login_shell_path_dirs(shell: &str, timeout: Duration) -> Vec<PathBuf> {
    let mut child = match Command::new(shell)
        .arg("-ilc")
        .arg("echo -n \"$PATH\"")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return Vec::new(),
    };

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Vec::new();
                }
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut output);
                }
                let trimmed = output.trim();
                if trimmed.is_empty() {
                    return Vec::new();
                }
                return env::split_paths(trimmed).collect();
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Vec::new();
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return Vec::new(),
        }
    }
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

    // --- login_shell_path_dirs -------------------------------------------

    #[cfg(unix)]
    // On overlayfs (Docker's default storage driver), a file that was just
    // written and chmod'd can transiently exec-fail with ETXTBSY
    // ("Text file busy") under concurrent `cargo test` threads — a known
    // overlayfs copy-up race, not anything about the write itself being
    // incomplete. An explicit `sync_all` before returning consistently
    // avoided it in practice (reproduced with a tight repeated-run loop).
    fn fake_shell(dir: &Path, name: &str, script: &str) -> PathBuf {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(script.as_bytes()).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn login_shell_path_dirs_parses_the_shells_reported_path() {
        let tmp = tempfile::tempdir().unwrap();
        let shell = fake_shell(
            tmp.path(),
            "fake-shell",
            "#!/bin/sh\necho -n \"/opt/custom/bin:/another/bin\"\n",
        );

        let result = login_shell_path_dirs(shell.to_str().unwrap(), Duration::from_secs(2));

        assert_eq!(
            result,
            vec![
                PathBuf::from("/opt/custom/bin"),
                PathBuf::from("/another/bin")
            ]
        );
    }

    #[test]
    fn login_shell_path_dirs_returns_empty_for_a_nonexistent_shell() {
        let result = login_shell_path_dirs("/no/such/shell", Duration::from_secs(2));
        assert!(result.is_empty());
    }

    #[test]
    fn login_shell_path_dirs_returns_empty_when_the_shell_exits_nonzero() {
        let tmp = tempfile::tempdir().unwrap();
        let shell = fake_shell(tmp.path(), "fake-shell", "#!/bin/sh\nexit 1\n");

        let result = login_shell_path_dirs(shell.to_str().unwrap(), Duration::from_secs(2));

        assert!(result.is_empty());
    }

    #[test]
    fn login_shell_path_dirs_times_out_instead_of_hanging() {
        let tmp = tempfile::tempdir().unwrap();
        // Simulates a broken/prompting rc script that never returns.
        let shell = fake_shell(tmp.path(), "fake-shell", "#!/bin/sh\nsleep 30\n");

        let start = Instant::now();
        let result = login_shell_path_dirs(shell.to_str().unwrap(), Duration::from_millis(200));
        let elapsed = start.elapsed();

        assert!(result.is_empty());
        assert!(
            elapsed < Duration::from_secs(2),
            "expected the timeout to cut this short, took {elapsed:?}"
        );
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
