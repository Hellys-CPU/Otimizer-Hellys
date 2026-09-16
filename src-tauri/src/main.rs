#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod engine;
mod models;
mod modules;

use db::Db;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path_resolver()
                .app_data_dir()
                .expect("não foi possível resolver o diretório de dados do aplicativo");
            let database = Db::open(app_data_dir).expect("falha ao inicializar o banco SQLite");
            app.manage(database);
            engine::game_watcher::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::list_optimizations,
            commands::apply_profile,
            commands::restore_profile,
            commands::restore_session,
            commands::apply_optimization,
            commands::restore_optimization,
            commands::list_sessions,
            commands::list_logs,
            commands::get_system_status,
            commands::list_top_processes,
            commands::list_detected_games,
            commands::register_game,
            commands::set_game_profile,
            commands::delete_game,
            commands::list_telemetry_comparisons,
            commands::get_telemetry_opt_in,
            commands::set_telemetry_opt_in,
            commands::clean_temp_files,
            commands::create_restore_point,
            commands::list_startup_apps,
            commands::list_services_diagnostic,
            commands::list_disks,
            commands::list_physical_disks,
            commands::list_gpus,
            commands::list_drivers,
            commands::get_current_power_scheme,
            commands::export_diagnostics,
            commands::capture_dpc_isr,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o SystemForge Optimizer");
}
