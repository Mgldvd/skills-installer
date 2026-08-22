use std::path::{Path, PathBuf};

use crate::domain::{DependencySource, DependencyStatus};
use crate::error::AppError;
use crate::platform::path_augment::augmented_search_paths;

/// No documented alias exists for the real `skills` CLI (confirmed against
/// the actual vercel-labs/skills tool this app targets) — kept as an
/// explicit, currently-empty extension point rather than folding "maybe
/// try another name" logic into the PATH-search step, so a future alias
/// only ever needs to be added here.
const KNOWN_ALIASES: &[&str] = &[];

/// What was found and how to invoke it: either the real executable
/// directly, or `npx --yes skills` as a fallback.
#[derive(Debug, Clone)]
pub struct ResolvedSkillsCli {
    pub program: String,
    pub leading_args: Vec<String>,
    pub source: DependencySource,
    pub executable_path: Option<String>,
}

/// Centralizes Skills CLI discovery per the documented order: 1) an
/// installed `skills` executable, 2) known aliases (see `KNOWN_ALIASES`),
/// 3) `npx --yes skills`, 4) a structured missing-dependency error. Never
/// trusts `std::env::var("PATH")` alone — GUI-launched Linux processes
/// (desktop entry, AppImage) commonly see a narrower `PATH` than an
/// interactive shell, per the official Tauri Linux distribution docs — so
/// resolution always searches `platform::path_augment::augmented_search_paths()`
/// instead.
#[derive(Default)]
pub struct DependencyResolver {
    /// Only ever set in tests (`with_search_paths`) so the full discovery
    /// order can be exercised against tempdir fixtures instead of this
    /// machine's real PATH — real usage always goes through `Default`.
    override_search_paths: Option<Vec<PathBuf>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            override_search_paths: Some(paths),
        }
    }

    /// Resolves the Skills CLI, then guards against the case a `skills` on
    /// PATH is actually *this app* under another name (e.g. a symlink named
    /// `skills` that points at a `skills-installer` binary/AppImage instead
    /// of the real vercel-labs/skills CLI — observed in the wild, and not
    /// caught by `is_application_launcher`'s path-identity check whenever
    /// the running process isn't itself the exact same file, such as a dev
    /// build). Confirmed by content instead of by path: this binary's own
    /// `--version` output always starts with `skills-installer` (see
    /// `cli::args::Cli`'s clap `name`), so any resolved executable that
    /// echoes the same self-identifying prefix is excluded and discovery
    /// retries the remaining candidates, falling through to aliases/npx/
    /// the missing-dependency error exactly as if it had never been found.
    pub async fn resolve(&self) -> Result<ResolvedSkillsCli, AppError> {
        let search_paths = self
            .override_search_paths
            .clone()
            .unwrap_or_else(augmented_search_paths);
        let mut excluded: Vec<PathBuf> = Vec::new();
        loop {
            let resolved = Self::resolve_with_paths_excluding(&search_paths, &excluded)?;
            if resolved.source == DependencySource::InstalledExecutable
                && self_identifies_as_installer(&resolved).await
            {
                excluded.push(PathBuf::from(&resolved.program));
                continue;
            }
            return Ok(resolved);
        }
    }

    #[cfg(test)]
    fn resolve_with_paths(search_paths: &[PathBuf]) -> Result<ResolvedSkillsCli, AppError> {
        Self::resolve_with_paths_excluding(search_paths, &[])
    }

    fn resolve_with_paths_excluding(
        search_paths: &[PathBuf],
        excluded: &[PathBuf],
    ) -> Result<ResolvedSkillsCli, AppError> {
        if let Some(path) = find_executable("skills", search_paths, excluded)
            .filter(|candidate| !is_application_launcher(candidate))
        {
            return Ok(ResolvedSkillsCli {
                program: path.to_string_lossy().to_string(),
                leading_args: Vec::new(),
                source: DependencySource::InstalledExecutable,
                executable_path: Some(path.to_string_lossy().to_string()),
            });
        }

        for alias in KNOWN_ALIASES {
            if let Some(path) = find_executable(alias, search_paths, excluded) {
                return Ok(ResolvedSkillsCli {
                    program: path.to_string_lossy().to_string(),
                    leading_args: Vec::new(),
                    source: DependencySource::InstalledExecutable,
                    executable_path: Some(path.to_string_lossy().to_string()),
                });
            }
        }

        if let Some(npx_path) = find_executable("npx", search_paths, excluded) {
            return Ok(ResolvedSkillsCli {
                program: npx_path.to_string_lossy().to_string(),
                leading_args: vec!["--yes".to_string(), "skills".to_string()],
                source: DependencySource::Npx,
                executable_path: Some(npx_path.to_string_lossy().to_string()),
            });
        }

        Err(AppError::MissingDependency(
            "The Skills CLI was not found. Install it with \"npm install -g skills\", or ensure \"npx\" is available on PATH."
                .to_string(),
        ))
    }

    pub async fn status(&self) -> DependencyStatus {
        match self.resolve().await {
            Ok(resolved) => {
                let version = probe_version(&resolved).await;
                DependencyStatus {
                    available: true,
                    source: resolved.source,
                    executable_path: resolved.executable_path,
                    version,
                    detail: None,
                }
            }
            Err(err) => DependencyStatus {
                available: false,
                source: DependencySource::Unavailable,
                executable_path: None,
                version: None,
                detail: Some(err.to_string()),
            },
        }
    }
}

fn is_application_launcher(candidate: &Path) -> bool {
    let candidate = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.to_path_buf());
    let current_exe = std::env::current_exe()
        .ok()
        .and_then(|path| path.canonicalize().ok());
    let app_image = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .and_then(|path| path.canonicalize().ok());

    current_exe.as_ref() == Some(&candidate) || app_image.as_ref() == Some(&candidate)
}

async fn self_identifies_as_installer(resolved: &ResolvedSkillsCli) -> bool {
    probe_version(resolved)
        .await
        .is_some_and(|version| version.to_lowercase().starts_with("skills-installer"))
}

async fn probe_version(resolved: &ResolvedSkillsCli) -> Option<String> {
    let output = tokio::process::Command::new(&resolved.program)
        .args(&resolved.leading_args)
        .arg("--version")
        .output()
        .await
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let text = if text.trim().is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        text
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn find_executable(name: &str, search_paths: &[PathBuf], excluded: &[PathBuf]) -> Option<PathBuf> {
    search_paths
        .iter()
        .map(|dir| dir.join(name))
        .find(|candidate| {
            candidate.is_file() && is_executable(candidate) && !excluded.contains(candidate)
        })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    // On overlayfs (Docker's default storage driver), a file just written
    // and chmod'd can transiently exec-fail with ETXTBSY ("Text file busy")
    // under concurrent `cargo test` threads — a known overlayfs copy-up
    // race, not an incomplete write. An explicit `sync_all` before
    // returning avoids it (see the identical fix + rationale in
    // `platform::path_augment`'s own `fake_shell` test helper).
    fn write_executable(path: &std::path::Path, script: &str) {
        use std::io::Write;
        let mut file = fs::File::create(path).unwrap();
        file.write_all(script.as_bytes()).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }

    fn make_executable(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        write_executable(&path, "#!/bin/sh\necho fake\n");
        path
    }

    #[test]
    fn prefers_direct_skills_executable_over_npx() {
        let tmp = tempfile::tempdir().unwrap();
        make_executable(tmp.path(), "skills");
        make_executable(tmp.path(), "npx");

        let resolved = DependencyResolver::resolve_with_paths(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(resolved.source, DependencySource::InstalledExecutable);
        assert!(resolved.leading_args.is_empty());
    }

    #[test]
    fn falls_back_to_npx_when_skills_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        make_executable(tmp.path(), "npx");

        let resolved = DependencyResolver::resolve_with_paths(&[tmp.path().to_path_buf()]).unwrap();
        assert_eq!(resolved.source, DependencySource::Npx);
        assert_eq!(
            resolved.leading_args,
            vec!["--yes".to_string(), "skills".to_string()]
        );
    }

    #[test]
    fn errors_with_missing_dependency_when_nothing_found() {
        let tmp = tempfile::tempdir().unwrap();
        let result = DependencyResolver::resolve_with_paths(&[tmp.path().to_path_buf()]);
        assert!(matches!(result, Err(AppError::MissingDependency(_))));
    }

    #[test]
    fn ignores_non_executable_files() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("skills"), "not executable").unwrap();
        let result = DependencyResolver::resolve_with_paths(&[tmp.path().to_path_buf()]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn probes_version_from_a_fake_executable() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("skills");
        write_executable(&path, "#!/bin/sh\necho 1.2.3\n");

        let resolved = ResolvedSkillsCli {
            program: path.to_string_lossy().to_string(),
            leading_args: vec![],
            source: DependencySource::InstalledExecutable,
            executable_path: Some(path.to_string_lossy().to_string()),
        };
        assert_eq!(probe_version(&resolved).await.as_deref(), Some("1.2.3"));
    }

    // --- self-identification guard --------------------------------------
    //
    // Reproduces a real-world setup found on a user's machine: a `skills`
    // executable on PATH that is actually a symlink to the skills-installer
    // app itself (not the real vercel-labs/skills CLI). `is_application_launcher`
    // only catches this when the candidate is byte-identical to the running
    // process's own exe / `$APPIMAGE` target, which doesn't hold for e.g. a
    // dev build or a stray/misdirected symlink — so `resolve()` must also
    // reject any candidate whose own `--version` output self-identifies as
    // `skills-installer` and keep searching instead.

    fn make_self_identifying_skills(dir: &Path) -> PathBuf {
        let path = dir.join("skills");
        write_executable(&path, "#!/bin/sh\necho 'skills-installer 0.1.0'\n");
        path
    }

    #[tokio::test]
    async fn resolve_skips_a_skills_executable_that_is_actually_this_app() {
        let tmp = tempfile::tempdir().unwrap();
        make_self_identifying_skills(tmp.path());
        make_executable(tmp.path(), "npx");

        let resolver = DependencyResolver::with_search_paths(vec![tmp.path().to_path_buf()]);
        let resolved = resolver.resolve().await.unwrap();

        assert_eq!(resolved.source, DependencySource::Npx);
    }

    #[tokio::test]
    async fn resolve_errors_when_the_only_skills_executable_is_this_app_and_no_npx() {
        let tmp = tempfile::tempdir().unwrap();
        make_self_identifying_skills(tmp.path());

        let resolver = DependencyResolver::with_search_paths(vec![tmp.path().to_path_buf()]);
        let result = resolver.resolve().await;

        assert!(matches!(result, Err(AppError::MissingDependency(_))));
    }

    #[tokio::test]
    async fn resolve_accepts_a_real_skills_executable() {
        let tmp = tempfile::tempdir().unwrap();
        make_executable(tmp.path(), "skills");

        let resolver = DependencyResolver::with_search_paths(vec![tmp.path().to_path_buf()]);
        let resolved = resolver.resolve().await.unwrap();

        assert_eq!(resolved.source, DependencySource::InstalledExecutable);
    }
}
