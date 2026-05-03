pub mod platform;
pub mod tools;
pub mod error;
pub mod models;
pub mod commands;
pub mod services;

#[cfg(target_os = "macos")]
const WINDOW_CORNER_RADIUS: f64 = 28.0;

#[cfg(target_os = "macos")]
fn apply_window_corner_radius(window: &tauri::WebviewWindow) {
    use objc::runtime::{Object, Sel, NO, YES};
    use objc::{class, msg_send, sel, sel_impl};

    let Ok(ns_window) = window.ns_window() else { return };
    let ns_window = ns_window as *mut Object;
    if ns_window.is_null() { return; }

    unsafe fn round_layer(view: *mut Object, radius: f64) {
        if view.is_null() { return; }
        let _: () = msg_send![view, setWantsLayer: YES];
        let layer: *mut Object = msg_send![view, layer];
        if layer.is_null() { return; }
        let _: () = msg_send![layer, setCornerRadius: radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];
    }

    // Drop the 1px AppKit stroke and chrome tint that AppKit traces along the
    // rounded path on the NSThemeFrame layer; without this a thin rim is
    // visible just outside the rounded corners.
    unsafe fn neutralize_layer_chrome(view: *mut Object) {
        if view.is_null() { return; }
        let layer: *mut Object = msg_send![view, layer];
        if layer.is_null() { return; }

        let _: () = msg_send![layer, setBorderWidth: 0.0_f64];
        let nil_color: *mut Object = std::ptr::null_mut();
        let _: () = msg_send![layer, setBorderColor: nil_color];

        let ns_color_cls = class!(NSColor);
        let clear_ns: *mut Object = msg_send![ns_color_cls, clearColor];
        let clear_cg: *mut Object = msg_send![clear_ns, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: clear_cg];
    }

    // Ensure AppKit views in the hierarchy are also transparent (not just the
    // NSWindow container), otherwise a white NSView/WKWebView background can
    // still show through even when the window itself is transparent.
    unsafe fn clear_view_background_recursive(view: *mut Object) {
        if view.is_null() { return; }

        unsafe fn responds_to(view: *mut Object, selector: Sel) -> bool {
            if view.is_null() { return false; }
            msg_send![view, respondsToSelector: selector]
        }

        let ns_color_cls = class!(NSColor);
        let clear_ns: *mut Object = msg_send![ns_color_cls, clearColor];

        if responds_to(view, sel!(setWantsLayer:)) {
            let _: () = msg_send![view, setWantsLayer: YES];
            let layer: *mut Object = msg_send![view, layer];
            if !layer.is_null() {
                let clear_cg: *mut Object = msg_send![clear_ns, CGColor];
                let _: () = msg_send![layer, setBackgroundColor: clear_cg];
            }
        }

        if responds_to(view, sel!(setBackgroundColor:)) {
            let _: () = msg_send![view, setBackgroundColor: clear_ns];
        }

        if responds_to(view, sel!(setDrawsBackground:)) {
            let _: () = msg_send![view, setDrawsBackground: NO];
        }

        if responds_to(view, sel!(subviews)) {
            let subviews: *mut Object = msg_send![view, subviews];
            if !subviews.is_null() {
                let count: usize = msg_send![subviews, count];
                for idx in 0..count {
                    let child: *mut Object = msg_send![subviews, objectAtIndex: idx];
                    clear_view_background_recursive(child);
                }
            }
        }
    }

    unsafe {
        let content_view: *mut Object = msg_send![ns_window, contentView];
        if content_view.is_null() { return; }
        round_layer(content_view, WINDOW_CORNER_RADIUS);

        // The contentView's superview is the NSWindow's frame view (NSThemeFrame).
        // Without rounding it too, its square corners bleed above the rounded
        // contentView and the system shadow traces the square frame.
        let frame_view: *mut Object = msg_send![content_view, superview];
        round_layer(frame_view, WINDOW_CORNER_RADIUS);

        neutralize_layer_chrome(content_view);
        neutralize_layer_chrome(frame_view);
        clear_view_background_recursive(frame_view);
        clear_view_background_recursive(content_view);

        // Re-assert window-level transparency in case AppKit re-derived a
        // non-clear backing color after we touched the frame_view's layer.
        let _: () = msg_send![ns_window, setOpaque: NO];
        let ns_color_cls = class!(NSColor);
        let clear_color: *mut Object = msg_send![ns_color_cls, clearColor];
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];

        // Force the system shadow to recompute from the new rounded shape.
        let _: () = msg_send![ns_window, setHasShadow: YES];
        let _: () = msg_send![ns_window, invalidateShadow];
    }
}

// Toggle a real macOS NSVisualEffectView (Sidebar material, BlendingMode =
// BehindWindow) behind the webview, so the chrome blurs the desktop instead
// of just lowering panel alpha. Implemented via the window-vibrancy crate to
// avoid hand-rolling NSRect / NSWindowOrderingMode FFI calls. Idempotent —
// re-applying the same material is a no-op; clearing when nothing is applied
// returns Err which we ignore.
#[cfg(target_os = "macos")]
pub(crate) fn set_vibrancy_visible(window: &tauri::WebviewWindow, visible: bool) {
    use window_vibrancy::{apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

    if visible {
        let _ = apply_vibrancy(
            window,
            NSVisualEffectMaterial::Sidebar,
            Some(NSVisualEffectState::FollowsWindowActiveState),
            Some(WINDOW_CORNER_RADIUS),
        );
    } else {
        let _ = clear_vibrancy(window);
    }

    // Re-recompute the system shadow against the new backing material.
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as *mut Object;
        if !ns_window.is_null() {
            unsafe { let _: () = msg_send![ns_window, invalidateShadow]; }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize log persistence (best-effort, non-fatal on error).
            match services::log_persist::init(app.handle()) {
                Ok(path) => {
                    eprintln!("[log_persist] Logging to {}", path.display());
                }
                Err(e) => {
                    eprintln!("[log_persist] Failed to initialize: {}", e);
                }
            }

            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    apply_window_corner_radius(&window);
                }
            }
            #[cfg(not(target_os = "macos"))]
            { let _ = app; }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::device::detect_device,
            commands::restore::get_restore_options,
            commands::restore::list_firmwares,
            commands::restore::check_ipsw_signing,
            commands::restore::cancel_ipsw_download,
            commands::restore::download_ipsw,
            commands::restore::verify_ipsw,
            commands::restore::prepare_ipsw,
            commands::restore::preview_restore_command,
            commands::restore::start_restore,
            commands::jailbreak::run_gaster,
            commands::jailbreak::run_kloader,
            commands::jailbreak::run_g1lbertjb,
            commands::jailbreak::run_evasi0n,
            commands::jailbreak::enter_pwndfu,
            commands::jailbreak::download_pwn_tool,
            commands::just_boot::list_just_boot_history,
            commands::just_boot::record_just_boot,
            commands::just_boot::forget_just_boot,
            commands::just_boot::prepare_and_just_boot,
            commands::firmware::extract_ipsw_component,
            commands::firmware::patch_iboot,
            commands::firmware::pack_img4,
            commands::firmware::repack_img3,
            commands::firmware::patch_kernel,
            commands::firmware::modify_ramdisk,
            commands::shsh::save_shsh_blob,
            commands::shsh::fetch_cydia_blobs,
            commands::shsh::dump_onboard_blob,
            commands::shsh::list_saved_blobs,
            commands::apps::list_installed_apps,
            commands::apps::install_ipa,
            commands::apps::uninstall_app,
            commands::data::create_backup,
            commands::data::restore_backup,
            commands::data::erase_device,
            commands::data::set_backup_encryption,
            commands::data::list_backups,
            commands::utilities::enter_recovery,
            commands::utilities::exit_recovery,
            commands::utilities::run_diagnostics_action,
            commands::utilities::pair_device,
            commands::utilities::run_activation_action,
            commands::utilities::export_device_info,
            commands::utilities::run_irecovery_commands,
            commands::utilities::clear_nvram,
            commands::utilities::start_syslog,
            commands::utilities::stop_syslog,
            commands::utilities::syslog_status,
            commands::utilities::get_log_file_path,
            commands::settings::get_app_settings,
            commands::settings::set_workspace_root,
            commands::settings::pick_workspace_root,
            commands::settings::complete_onboarding,
            commands::settings::ensure_workspace_layout,
            commands::settings::reveal_workspace,
            commands::settings::set_window_shadow,
            commands::settings::set_glass_chrome,
            commands::trollstore::prepare_trollstore_assets,
            commands::trollstore::check_trollstore_eligibility,
            commands::updates::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
