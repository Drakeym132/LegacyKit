use crate::error::AppError;
use crate::models::settings::{AppSettings, SetWorkspaceRootRequest, WorkspacePaths};
use crate::services::{app_settings, workspace};
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

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
    if path.is_empty() {
        return Err(AppError::Parse("Workspace path is required".to_string()));
    }

    let mut settings = app_settings::load(&app)?;
    let root = PathBuf::from(path);
    let layout = workspace::WorkspaceLayout::from_root(root.clone());
    layout.ensure_layout()?;

    settings.workspace_root = Some(root);
    app_settings::save(&app, &settings)?;
    Ok(settings)
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
        .map_err(|err| AppError::CommandFailed(format!("Failed to toggle glass chrome on main thread: {err}")))?;
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
        .map_err(|err| AppError::CommandFailed(format!("Failed to run shadow toggle on main thread: {err}")))?;
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
