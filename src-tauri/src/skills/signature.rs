use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::AppError;

/// Rust port of `sign-folder` (the standalone Python script this project
/// used to depend on being present on `$PATH`): a deterministic,
/// content-based `.signature` file per skill folder, so an installed Local
/// skill can be compared against its catalog source to tell whether the
/// catalog has changed since install — see `SkillsService::check_local_updates`.
///
/// Byte-for-byte compatible with the original: same `sign-folder-v1` digest
/// (SHA-256 over every entry's type, path, and content, in canonical sorted
/// order) and the same two-line `id-<uuid v4>` / `sign-<hex>` format, so
/// catalogs already signed by the Python tool on another machine keep
/// comparing correctly against skills this app signs itself.
///
/// ponytail: skips the original's Unicode NFC normalization of file names
/// (`unicodedata.normalize`) — skill folder names are ASCII slugs in
/// practice, where NFC is a no-op anyway. Revisit if a catalog ever uses
/// non-ASCII folder names and signatures stop matching across machines.
const FORMAT: &[u8] = b"sign-folder-v1";
const SIGNATURE_NAME: &str = ".signature";
const ID_PREFIX: &str = "id-";
const OUTPUT_PREFIX: &str = "sign-";
const BUFFER_SIZE: usize = 1024 * 1024;

fn add_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

/// Depth-first, sorted-ascending walk of `dir`'s contents, folding every
/// entry into `hasher` in the same order `sign-folder`'s iterative stack
/// walk produces (recurse fully into each subdirectory, in name order,
/// before moving to the next sibling) — see module docs.
fn hash_dir(
    dir: &Path,
    relative_prefix: &[u8],
    skip_signature: bool,
    hasher: &mut Sha256,
) -> Result<(), AppError> {
    let read_err =
        |e: std::io::Error| AppError::Io(format!("cannot read directory {}: {e}", dir.display()));
    let mut entries: Vec<(Vec<u8>, std::fs::DirEntry)> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(read_err)? {
        let entry = entry.map_err(read_err)?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            AppError::Validation(format!(
                "path is not valid UTF-8: {}",
                entry.path().display()
            ))
        })?;
        if skip_signature && name == SIGNATURE_NAME {
            continue;
        }
        entries.push((name.as_bytes().to_vec(), entry));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, entry) in entries {
        let relative_path = if relative_prefix.is_empty() {
            name
        } else {
            [relative_prefix, b"/", name.as_slice()].concat()
        };
        let metadata = std::fs::symlink_metadata(entry.path())
            .map_err(|e| AppError::Io(format!("cannot inspect {}: {e}", entry.path().display())))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            add_field(hasher, b"directory");
            add_field(hasher, &relative_path);
            hash_dir(&entry.path(), &relative_path, false, hasher)?;
        } else if file_type.is_file() {
            add_field(hasher, b"file");
            add_field(hasher, &relative_path);
            add_field(hasher, &metadata.len().to_be_bytes());
            let mut file = std::fs::File::open(entry.path()).map_err(|e| {
                AppError::Io(format!("cannot read file {}: {e}", entry.path().display()))
            })?;
            let mut buffer = vec![0u8; BUFFER_SIZE];
            loop {
                let read = file.read(&mut buffer).map_err(|e| {
                    AppError::Io(format!("cannot read file {}: {e}", entry.path().display()))
                })?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
        } else if file_type.is_symlink() {
            add_field(hasher, b"symlink");
            add_field(hasher, &relative_path);
            let target = std::fs::read_link(entry.path()).map_err(|e| {
                AppError::Io(format!(
                    "cannot read symlink {}: {e}",
                    entry.path().display()
                ))
            })?;
            let target = target.to_str().ok_or_else(|| {
                AppError::Validation(format!(
                    "symlink target is not valid UTF-8: {}",
                    entry.path().display()
                ))
            })?;
            add_field(hasher, target.as_bytes());
        } else {
            return Err(AppError::Validation(format!(
                "unsupported filesystem entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn folder_digest(root: &Path) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    add_field(&mut hasher, FORMAT);
    hash_dir(root, b"", true, &mut hasher)?;
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Reads the persistent UUID off an existing `.signature` file, so
/// re-signing a folder that already has one keeps the same id (matching
/// `sign-folder`'s default, non-`--new-id` behavior). Any missing or
/// unrecognized file (not present, legacy hash-only format, corrupt)
/// quietly yields `None` — the caller then mints a fresh one — rather than
/// failing the whole re-sign the way the original tool's strict `--verify`
/// mode would; this port only ever signs, never verifies.
fn read_folder_id(signature_path: &Path) -> Option<Uuid> {
    let content = std::fs::read_to_string(signature_path).ok()?;
    let first_line = content.lines().next()?;
    let id = first_line.strip_prefix(ID_PREFIX)?;
    Uuid::parse_str(id).ok()
}

/// (Re)computes and atomically writes `root`'s `.signature` file, preserving
/// its existing persistent UUID when there is one.
pub fn sign_folder(root: &Path) -> Result<(), AppError> {
    let signature_path = root.join(SIGNATURE_NAME);
    let folder_id = read_folder_id(&signature_path).unwrap_or_else(Uuid::new_v4);
    let digest = folder_digest(root)?;
    let content = format!("{ID_PREFIX}{folder_id}\n{OUTPUT_PREFIX}{digest}\n");

    let mut tmp = tempfile::NamedTempFile::new_in(root)?;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(&signature_path).map_err(|e| {
        AppError::Io(format!(
            "failed to finalize write to {}: {}",
            signature_path.display(),
            e.error
        ))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_signature(dir: &Path) -> String {
        std::fs::read_to_string(dir.join(SIGNATURE_NAME)).unwrap()
    }

    #[test]
    fn signs_a_folder_with_the_expected_two_line_format() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("SKILL.md"), b"hello").unwrap();

        sign_folder(tmp.path()).unwrap();

        let content = read_signature(tmp.path());
        let mut lines = content.lines();
        assert!(lines.next().unwrap().starts_with(ID_PREFIX));
        assert!(lines.next().unwrap().starts_with(OUTPUT_PREFIX));
        assert!(lines.next().is_none());
    }

    #[test]
    fn same_content_produces_the_same_digest() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("SKILL.md"), b"same bytes").unwrap();
        std::fs::write(b.path().join("SKILL.md"), b"same bytes").unwrap();

        sign_folder(a.path()).unwrap();
        sign_folder(b.path()).unwrap();

        let sig_a = read_signature(a.path());
        let sig_b = read_signature(b.path());
        let hash_of = |s: &str| s.lines().nth(1).unwrap().to_string();
        assert_eq!(hash_of(&sig_a), hash_of(&sig_b));
        // Independent folders never signed before get independent ids.
        assert_ne!(sig_a.lines().next(), sig_b.lines().next());
    }

    #[test]
    fn changed_content_produces_a_different_digest() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("SKILL.md"), b"v1").unwrap();
        sign_folder(tmp.path()).unwrap();
        let before = read_signature(tmp.path());

        std::fs::write(tmp.path().join("SKILL.md"), b"v2").unwrap();
        sign_folder(tmp.path()).unwrap();
        let after = read_signature(tmp.path());

        assert_ne!(before, after);
    }

    #[test]
    fn re_signing_preserves_the_existing_persistent_id() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("SKILL.md"), b"v1").unwrap();
        sign_folder(tmp.path()).unwrap();
        let id_before = read_signature(tmp.path())
            .lines()
            .next()
            .unwrap()
            .to_string();

        std::fs::write(tmp.path().join("SKILL.md"), b"v2").unwrap();
        sign_folder(tmp.path()).unwrap();
        let id_after = read_signature(tmp.path())
            .lines()
            .next()
            .unwrap()
            .to_string();

        assert_eq!(id_before, id_after);
    }

    #[test]
    fn renaming_a_file_changes_the_digest() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.md"), b"content").unwrap();
        sign_folder(tmp.path()).unwrap();
        let before = read_signature(tmp.path());

        std::fs::rename(tmp.path().join("a.md"), tmp.path().join("b.md")).unwrap();
        sign_folder(tmp.path()).unwrap();
        let after = read_signature(tmp.path());

        assert_ne!(before.lines().nth(1), after.lines().nth(1));
    }

    // Not run by default (needs the original Python `sign-folder` — a
    // machine-local dev tool this repo can't depend on being installed, or
    // on `$PATH` under that exact name) — a one-off way to re-confirm byte
    // compatibility by hand: point `SIGN_FOLDER_BIN` at it (a path, or a
    // bare command already on `$PATH`) and run
    // `cargo test -- --ignored cross_checks_against_the_original_python_tool`.
    #[test]
    #[ignore]
    fn cross_checks_against_the_original_python_tool() {
        let Ok(sign_folder_bin) = std::env::var("SIGN_FOLDER_BIN") else {
            eprintln!(
                "skipping: set SIGN_FOLDER_BIN to the original sign-folder tool to run this check"
            );
            return;
        };
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("SKILL.md"), b"hello world\n").unwrap();
        std::fs::write(tmp.path().join("sub/file.txt"), b"nested content\n").unwrap();

        let python = std::process::Command::new(sign_folder_bin)
            .arg(tmp.path())
            .output()
            .expect("failed to run SIGN_FOLDER_BIN");
        assert!(python.status.success());
        let python_signature = read_signature(tmp.path());

        std::fs::remove_file(tmp.path().join(SIGNATURE_NAME)).unwrap();
        sign_folder(tmp.path()).unwrap();
        let rust_signature = read_signature(tmp.path());

        assert_eq!(
            python_signature.lines().nth(1),
            rust_signature.lines().nth(1),
            "digest line must match the original tool's output byte-for-byte"
        );
    }

    #[test]
    fn nested_directories_affect_the_digest() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("sub/file.md"), b"nested").unwrap();
        sign_folder(tmp.path()).unwrap();
        let nested_hash = read_signature(tmp.path())
            .lines()
            .nth(1)
            .unwrap()
            .to_string();

        let flat = tempfile::tempdir().unwrap();
        std::fs::write(flat.path().join("file.md"), b"nested").unwrap();
        sign_folder(flat.path()).unwrap();
        let flat_hash = read_signature(flat.path())
            .lines()
            .nth(1)
            .unwrap()
            .to_string();

        assert_ne!(nested_hash, flat_hash);
    }
}
