use crate::error::AppError;
use crate::models::settings::{AppSettings, SetWorkspaceRootRequest, WorkspacePaths};
use crate::services::{app_settings, workspace};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;

#[tauri::command]
pub async fn get_app_settings(app: AppHandle) -> Result<AppSettings, AppError> {
    app_settings::load(&app)
}

#[tauri::command]
pub async fn set_workspace_root(
    app: AppHandle,
    request: SetWorkspaceRootRequest,
) -> Result<AppSettings, AppError> {
    let path = request.path.trim();

    validate_workspace_path(path)?;

    let mut settings = app_settings::load(&app)?;
    let root = PathBuf::from(path);
    let layout = workspace::WorkspaceLayout::from_root(root.clone());
    layout.ensure_layout()?;

    settings.workspace_root = Some(root);
    app_settings::save(&app, &settings)?;
    Ok(settings)
}

/// Validates that a workspace path can be created.
/// - Rejects empty strings (after trimming)
/// - Checks that parent directory exists and is writable
/// - Canonicalizes if the path exists; otherwise leaves as-is for first-run setup
fn validate_workspace_path(path: &str) -> Result<(), AppError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::Parse("Workspace path cannot be empty".into()));
    }

    let root = PathBuf::from(path);

    // If the path already exists, canonicalize it
    let root = if root.exists() {
        root.canonicalize().unwrap_or(root)
    } else {
        root
    };

    // Check parent directory
    let parent = root.parent().ok_or_else(|| {
        AppError::Parse(format!(
            "Cannot create workspace at {path}: parent directory does not exist or is not writable"
        ))
    })?;

    // Parent must exist and be a directory
    let metadata = fs::metadata(parent).map_err(|_| {
        AppError::Parse(format!(
            "Cannot create workspace at {path}: parent directory does not exist or is not writable"
        ))
    })?;

    if !metadata.is_dir() {
        return Err(AppError::Parse(format!(
            "Cannot create workspace at {path}: parent directory does not exist or is not writable"
        )));
    }

    // Test write permission by creating and removing a temp marker file
    let marker_name = format!(".legacykit-write-test-{}", std::process::id());
    let marker_path = parent.join(&marker_name);
    let write_ok = fs::write(&marker_path, b"").is_ok();
    if write_ok {
        // Suppress errors on cleanup
        let _ = fs::remove_file(&marker_path);
    }

    if !write_ok {
        return Err(AppError::Parse(format!(
            "Cannot create workspace at {path}: parent directory does not exist or is not writable"
        )));
    }

    Ok(())
}

#[tauri::command]
pub async fn complete_onboarding(app: AppHandle) -> Result<AppSettings, AppError> {
    let mut settings = app_settings::load(&app)?;
    if settings.workspace_root.is_none() {
        return Err(AppError::Parse(
            "Workspace must be set before onboarding can be completed".to_string(),
        ));
    }
    settings.onboarded = true;
    app_settings::save(&app, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn ensure_workspace_layout(app: AppHandle) -> Result<WorkspacePaths, AppError> {
    let layout = workspace::get_layout(&app)?;
    layout.ensure_layout()?;
    Ok(layout_paths(&layout))
}

#[tauri::command]
pub async fn reveal_workspace(app: AppHandle) -> Result<(), AppError> {
    let layout = workspace::get_layout(&app)?;
    let root = layout.root();
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(root).status()?
    } else if cfg!(target_os = "windows") {
        Command::new("explorer").arg(root).status()?
    } else {
        Command::new("xdg-open").arg(root).status()?
    };

    if !status.success() {
        return Err(AppError::CommandFailed(
            "Failed to reveal workspace folder".to_string(),
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn pick_workspace_root(_app: AppHandle) -> Result<Option<String>, AppError> {
    Ok(None)
}

#[tauri::command]
pub async fn set_glass_chrome(app: AppHandle, enabled: bool) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        let app_handle = app.clone();
        app.run_on_main_thread(move || {
            if let Some(window) = app_handle.get_webview_window("main") {
                crate::set_vibrancy_visible(&window, enabled);
            }
        })
        .map_err(|err| {
            AppError::CommandFailed(format!(
                "Failed to toggle glass chrome on main thread: {err}"
            ))
        })?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, enabled);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_window_shadow(app: AppHandle, enabled: bool) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

        let app_handle = app.clone();
        app.run_on_main_thread(move || {
            if let Some(window) = app_handle.get_webview_window("main") {
                if let Ok(ns_window) = window.ns_window() {
                    let ns_window = ns_window as *mut Object;
                    if !ns_window.is_null() {
                        unsafe {
                            let _: () = msg_send![ns_window, setHasShadow: enabled];
                            let _: () = msg_send![ns_window, invalidateShadow];
                        }
                    }
                }
            }
        })
        .map_err(|err| {
            AppError::CommandFailed(format!("Failed to run shadow toggle on main thread: {err}"))
        })?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, enabled);
    }

    Ok(())
}

fn layout_paths(layout: &workspace::WorkspaceLayout) -> WorkspacePaths {
    WorkspacePaths {
        root: layout.root().to_string_lossy().to_string(),
        ipsw: layout.ipsw_dir(None).to_string_lossy().to_string(),
        ipsw_custom: layout.custom_ipsw_dir(None).to_string_lossy().to_string(),
        shsh: layout.shsh_dir(None).to_string_lossy().to_string(),
        extracted: layout
            .extracted_dir(None, None)
            .to_string_lossy()
            .to_string(),
        ssh_binaries: layout.ssh_binaries_dir().to_string_lossy().to_string(),
        backups: layout.backups_dir(None).to_string_lossy().to_string(),
        logs: layout.logs_dir().to_string_lossy().to_string(),
        tmp: layout.tmp_dir().to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_string_returns_parse_error() {
        let result = validate_workspace_path("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Parse(msg) => assert_eq!(msg, "Workspace path cannot be empty"),
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn whitespace_only_returns_parse_error() {
        let result = validate_workspace_path("   ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Parse(msg) => assert_eq!(msg, "Workspace path cannot be empty"),
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn non_existent_parent_returns_parse_error() {
        let result = validate_workspace_path("/nonexistent/path/to/workspace");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Parse(msg) => {
                assert!(msg.contains("Cannot create workspace at"));
                assert!(msg.contains("parent directory does not exist or is not writable"));
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn valid_temp_dir_succeeds() {
        let temp_base = std::env::temp_dir();
        let unique_subpath = format!(
            "legacykit-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        );
        let workspace_path = temp_base.join(&unique_subpath);
        let path_str = workspace_path.to_string_lossy().to_string();

        let result = validate_workspace_path(&path_str);
        assert!(
            result.is_ok(),
            "Expected validation to succeed for temp dir path"
        );

        // Cleanup if the test created anything
        let _ = fs::remove_dir_all(&workspace_path);
    }
}
