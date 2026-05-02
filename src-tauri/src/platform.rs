use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Resolves the path to a bundled sidecar binary based on the current platform and architecture.
///
/// Looks first in `binaries/<os>/<arch>/<name>`, then falls back to `binaries/<os>/<name>` (flat).
pub fn resolve_binary_path(app: &AppHandle, binary_name: &str) -> Result<PathBuf, String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform_dir = match os {
        "macos" => "macos",
        "linux" => "linux",
        _ => return Err(format!("Unsupported operating system: {}", os)),
    };

    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let base_bin_dir = resource_path.join("binaries").join(platform_dir);

    let candidates = build_candidate_paths(&base_bin_dir, arch, binary_name);

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(format!(
        "Binary '{}' not found for {} {} (tried: {})",
        binary_name,
        os,
        arch,
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// Pure helper: builds the list of paths to try for a given (base_bin_dir, arch, binary_name).
/// First the arch-specific subdir, then the flat fallback.
fn build_candidate_paths(base_bin_dir: &Path, arch: &str, binary_name: &str) -> Vec<PathBuf> {
    let arch_dir = match arch {
        "aarch64" => Some("arm64"),
        "x86_64" => Some("x86_64"),
        _ => None,
    };

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(a) = arch_dir {
        candidates.push(base_bin_dir.join(a).join(binary_name));
    }
    candidates.push(base_bin_dir.join(binary_name));
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_include_arch_dir_first_then_flat() {
        let base = PathBuf::from("/tmp/binaries/linux");
        let v = build_candidate_paths(&base, "aarch64", "tsschecker");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], PathBuf::from("/tmp/binaries/linux/arm64/tsschecker"));
        assert_eq!(v[1], PathBuf::from("/tmp/binaries/linux/tsschecker"));
    }

    #[test]
    fn candidates_x86_64_uses_x86_64_subdir() {
        let base = PathBuf::from("/tmp/binaries/linux");
        let v = build_candidate_paths(&base, "x86_64", "ideviceinfo");
        assert_eq!(v[0], PathBuf::from("/tmp/binaries/linux/x86_64/ideviceinfo"));
        assert_eq!(v[1], PathBuf::from("/tmp/binaries/linux/ideviceinfo"));
    }

    #[test]
    fn candidates_unknown_arch_falls_back_to_flat_only() {
        let base = PathBuf::from("/tmp/binaries/macos");
        let v = build_candidate_paths(&base, "powerpc", "ipsw");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], PathBuf::from("/tmp/binaries/macos/ipsw"));
    }

    #[test]
    fn candidates_macos_arm64_includes_arch_dir_then_flat() {
        let base = PathBuf::from("/tmp/binaries/macos");
        let v = build_candidate_paths(&base, "aarch64", "irecovery");
        assert_eq!(v[0], PathBuf::from("/tmp/binaries/macos/arm64/irecovery"));
        assert_eq!(v[1], PathBuf::from("/tmp/binaries/macos/irecovery"));
    }
}
