pub mod platform;
pub mod tools;
pub mod error;
pub mod models;
pub mod commands;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            tools::runner::execute_tool,
            tools::runner::execute_idevice_info,
            tools::runner::execute_irecovery,
            commands::device::detect_device,
            commands::restore::get_restore_options,
            commands::restore::download_ipsw,
            commands::restore::verify_ipsw,
            commands::restore::prepare_ipsw,
            commands::restore::preview_restore_command,
            commands::restore::start_restore,
            commands::jailbreak::run_gaster,
            commands::jailbreak::run_kloader,
            commands::jailbreak::run_g1lbertjb,
            commands::jailbreak::run_evasi0n,
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
            commands::trollstore::prepare_trollstore_assets,
            commands::trollstore::check_trollstore_eligibility,
            commands::updates::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
