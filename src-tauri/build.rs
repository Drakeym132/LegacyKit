fn main() {
    // On macOS, ad-hoc sign sidecar binaries and strip quarantine flags
    // ONLY in release builds. This prevents "Apple could not verify ... is free of malware"
    // popups when running unsigned third-party tools (gaster, ipwnder, etc.).
    // In dev builds, we skip this to avoid modifying binaries on every incremental compile.
    #[cfg(target_os = "macos")]
    {
        let profile = std::env::var("PROFILE").unwrap_or_default();
        if profile == "release" {
            sign_sidecars_macos();
        }
    }

    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn sign_sidecars_macos() {
    use std::env;
    use std::path::Path;

    // The binaries directory is relative to the crate root (src-tauri/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let binaries_dir = Path::new(&manifest_dir).join("binaries").join("macos");

    if !binaries_dir.exists() {
        println!("cargo:warning=Sidecar binaries directory not found: {:?}", binaries_dir);
        return;
    }

    // Find all executable files under binaries/macos/<arch>/
    let mut signed_count = 0;
    if let Ok(entries) = std::fs::read_dir(&binaries_dir) {
        for arch_entry in entries.flatten() {
            let arch_path = arch_entry.path();
            if arch_path.is_dir() {
                if let Ok(binaries) = std::fs::read_dir(&arch_path) {
                    for binary_entry in binaries.flatten() {
                        let binary_path = binary_entry.path();
                        if is_executable(&binary_path) {
                            if ad_hoc_sign(&binary_path) {
                                signed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("cargo:warning=Ad-hoc signed {} sidecar binaries on macOS (release build)", signed_count);
}

#[cfg(target_os = "macos")]
fn is_executable(path: &std::path::Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Skip directories and non-files
    if !path.is_file() {
        return false;
    }

    // Check if any execute bit is set
    if let Ok(metadata) = fs::metadata(path) {
        let mode = metadata.permissions().mode();
        return (mode & 0o111) != 0;
    }

    false
}

#[cfg(target_os = "macos")]
fn ad_hoc_sign(path: &std::path::Path) -> bool {
    use std::process::Command;

    // Strip quarantine xattr first (in case it was set)
    // Ignore errors — the xattr may not be present
    let _ = Command::new("xattr")
        .arg("-d")
        .arg("com.apple.quarantine")
        .arg(path)
        .status();

    // Ad-hoc sign the binary (no identity, just "-" for ad-hoc)
    let result = Command::new("codesign")
        .arg("--force")
        .arg("--sign")
        .arg("-")
        .arg("--timestamp=none")
        .arg(path)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("cargo:warning=Failed to sign {:?}: {}", path, stderr);
                false
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to run codesign for {:?}: {}", path, e);
            false
        }
    }
}
