use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckRequest {
    pub repo: String,
    pub current_version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub current: String,
    pub latest: String,
    pub release_url: Option<String>,
    pub update_available: bool,
}

#[tauri::command]
pub async fn check_for_updates(
    request: UpdateCheckRequest,
) -> Result<UpdateCheckResult, AppError> {
    let repo = request.repo.trim();
    if repo.is_empty() || !repo.contains('/') {
        return Err(AppError::Parse(
            "Repo must be in 'owner/name' form".into(),
        ));
    }
    let current = request.current_version.trim().to_string();
    if current.is_empty() {
        return Err(AppError::Parse("Current version is required".into()));
    }

    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let curl = which("curl")
        .ok_or_else(|| AppError::CommandFailed("curl is required to query GitHub".into()))?;
    let output = Command::new(&curl)
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-A",
            "legacykit",
            &api_url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::CommandFailed(if stderr.is_empty() {
            format!("curl exited with {}", output.status)
        } else {
            stderr
        }));
    }
    let body = String::from_utf8_lossy(&output.stdout);
    let latest = parse_json_string(&body, "tag_name")
        .ok_or_else(|| AppError::Parse("tag_name not found in response".into()))?;
    let release_url = parse_json_string(&body, "html_url");

    let cmp = compare_versions(&current, &latest);
    Ok(UpdateCheckResult {
        current,
        latest,
        release_url,
        update_available: cmp.is_lt(),
    })
}

/// Loose comparison: strips a leading 'v', splits on dots, compares numeric segments
/// where possible, falls back to lexicographic.
fn compare_versions(current: &str, latest: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let a = strip_v(current);
    let b = strip_v(latest);
    let parts_a: Vec<&str> = a.split('.').collect();
    let parts_b: Vec<&str> = b.split('.').collect();
    let len = parts_a.len().max(parts_b.len());
    for i in 0..len {
        let pa = parts_a.get(i).copied().unwrap_or("0");
        let pb = parts_b.get(i).copied().unwrap_or("0");
        let na = pa.parse::<u64>().ok();
        let nb = pb.parse::<u64>().ok();
        let ord = match (na, nb) {
            (Some(x), Some(y)) => x.cmp(&y),
            _ => pa.cmp(pb),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn strip_v(s: &str) -> &str {
    let trimmed = s.trim();
    trimmed.strip_prefix('v').unwrap_or(trimmed)
}

fn parse_json_string(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = body.find(&needle)?;
    let after = &body[start + needle.len()..];
    let colon = after.find(':')?;
    let rest = &after[colon + 1..];
    let q1 = rest.find('"')?;
    let after_q1 = &rest[q1 + 1..];
    let q2 = after_q1.find('"')?;
    let value = &after_q1[..q2];
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn newer_latest_returns_less() {
        assert_eq!(compare_versions("1.0.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_versions("v1.0.0", "v1.1.0"), Ordering::Less);
        assert_eq!(compare_versions("0.9.9", "1.0.0"), Ordering::Less);
    }

    #[test]
    fn equal_versions() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("v1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn older_latest() {
        assert_eq!(compare_versions("2.0.0", "1.5.0"), Ordering::Greater);
    }

    #[test]
    fn missing_segments_treated_as_zero() {
        assert_eq!(compare_versions("1.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_versions("1", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn parse_html_url() {
        let body = r#"{"tag_name":"1.2.3","html_url":"https://github.com/x/y/releases/tag/1.2.3"}"#;
        assert_eq!(parse_json_string(body, "tag_name").as_deref(), Some("1.2.3"));
        assert_eq!(
            parse_json_string(body, "html_url").as_deref(),
            Some("https://github.com/x/y/releases/tag/1.2.3"),
        );
    }
}
