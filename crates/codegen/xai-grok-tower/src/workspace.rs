//! Workspace path policy helpers (canonicalization / symlink fail-closed).

use std::path::{Path, PathBuf};

use crate::RuntimeError;

/// Canonicalize and require the resolved path to remain under an allowed root.
/// Fail closed if resolution fails or escapes.
pub fn authorize_workspace_path(
    workspace_root: &Path,
    allowed_root: &Path,
) -> Result<PathBuf, RuntimeError> {
    let canonical = dunce::canonicalize(workspace_root).map_err(|e| RuntimeError {
        code: "invalid_workspace",
        message: format!("workspace resolution failed: {e}"),
    })?;
    let allowed = dunce::canonicalize(allowed_root).map_err(|e| RuntimeError {
        code: "invalid_workspace",
        message: format!("allowed root resolution failed: {e}"),
    })?;
    if !canonical.starts_with(&allowed) {
        return Err(RuntimeError {
            code: "invalid_workspace",
            message: "workspace escapes authorized root".into(),
        });
    }
    Ok(canonical)
}

/// Detect a path that no longer resolves to the previously authorized target
/// (symlink swap / TOCTOU characterization).
pub fn assert_workspace_stable(
    path: &Path,
    previously_authorized: &Path,
) -> Result<(), RuntimeError> {
    let now = dunce::canonicalize(path).map_err(|e| RuntimeError {
        code: "invalid_workspace",
        message: format!("workspace re-resolution failed: {e}"),
    })?;
    if now != previously_authorized {
        return Err(RuntimeError {
            code: "invalid_workspace",
            message: "workspace resolution changed after authorization".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod workspace_symlink_tests {
    use super::*;
    use std::fs;

    #[test]
    fn workspace_symlink_escape_is_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let allowed = temp.path().join("allowed");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&allowed).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let good = allowed.join("proj");
        fs::create_dir_all(&good).unwrap();
        let authorized = authorize_workspace_path(&good, &allowed).unwrap();
        assert!(authorized.starts_with(allowed.canonicalize().unwrap()));

        // Symlink inside allowed pointing outside must fail.
        let link = allowed.join("escape");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            let err = authorize_workspace_path(&link, &allowed).unwrap_err();
            assert_eq!(err.code, "invalid_workspace");
        }

        assert_workspace_stable(&good, &authorized).unwrap();
    }
}
