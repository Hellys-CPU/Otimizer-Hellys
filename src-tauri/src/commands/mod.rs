use crate::db::Db;
use crate::engine::profile as profile_engine;
use crate::models::{ExecutionLogEntry, ExecutionSession, Optimization, Profile, SystemStatus};
use tauri::State;

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub fn list_profiles(db: State<Db>) -> CmdResult<Vec<Profile>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, nome, descricao, icone, cor, nivel_agressividade, requer_administrador, ultima_aplicacao, ativo
             FROM profiles ORDER BY nome",
        )
        .map_err(|e| e.to_string())?;

    let profiles = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            Ok((
                id,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, bool>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, bool>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|(id, nome, descricao, icone, cor, nivel, requer_admin, ultima, ativo)| {
            let mut ids_stmt = conn
                .prepare("SELECT optimization_id FROM profile_optimizations WHERE profile_id = ?1 AND proibido = 0")
                .unwrap();
            let otimizacoes_ids: Vec<String> = ids_stmt
                .query_map([&id], |r| r.get(0))
                .unwrap()
                .filter_map(Result::ok)
                .collect();

            let mut proibidas_stmt = conn
                .prepare("SELECT optimization_id FROM profile_optimizations WHERE profile_id = ?1 AND proibido = 1")
                .unwrap();
            let otimizacoes_proibidas_ids: Vec<String> = proibidas_stmt
                .query_map([&id], |r| r.get(0))
                .unwrap()
                .filter_map(Result::ok)
                .collect();

            Profile {
                id,
                nome,
                descricao,
                icone,
                cor,
                nivel_agressividade: nivel,
                otimizacoes_ids,
                otimizacoes_proibidas_ids,
                requer_administrador: requer_admin,
                ultima_aplicacao: ultima,
                ativo,
            }
        })
        .collect();

    Ok(profiles)
}

#[tauri::command]
pub fn list_optimizations(db: State<Db>) -> CmdResult<Vec<Optimization>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, nome, categoria, descricao, beneficio_esperado, risco, requer_reinicializacao,
                    requer_administrador, valor_atual, valor_recomendado, fonte_tecnica, ativo
             FROM optimizations ORDER BY categoria, nome",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(Optimization {
                id: row.get(0)?,
                nome: row.get(1)?,
                categoria: row.get(2)?,
                descricao: row.get(3)?,
                beneficio_esperado: row.get(4)?,
                risco: row.get(5)?,
                requer_reinicializacao: row.get(6)?,
                requer_administrador: row.get(7)?,
                valor_atual: row.get(8)?,
                valor_recomendado: row.get(9)?,
                fonte_tecnica: row.get(10)?,
                ativo: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    Ok(items)
}

#[tauri::command]
pub fn apply_profile(db: State<Db>, profile_id: String) -> CmdResult<ExecutionSession> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    profile_engine::apply_profile(&conn, &profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_profile(db: State<Db>, profile_id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    profile_engine::restore_profile(&conn, &profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_session(db: State<Db>, session_id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    profile_engine::restore_session(&conn, &session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_optimization(db: State<Db>, optimization_id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    profile_engine::apply_single_optimization(&conn, &optimization_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_optimization(db: State<Db>, optimization_id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    profile_engine::restore_single_optimization(&conn, &optimization_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sessions(db: State<Db>) -> CmdResult<Vec<ExecutionSession>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, profile_id, started_at, finished_at, status FROM execution_sessions ORDER BY started_at DESC")
        .map_err(|e| e.to_string())?;
    let sessions = stmt
        .query_map([], |row| {
            Ok(ExecutionSession {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();
    Ok(sessions)
}

#[tauri::command]
pub fn list_logs(db: State<Db>, session_id: Option<String>) -> CmdResult<Vec<ExecutionLogEntry>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let query = "SELECT id, session_id, command, user_name, timestamp, result, error_message, return_code, requires_reboot
                 FROM execution_logs WHERE (?1 IS NULL OR session_id = ?1) ORDER BY timestamp DESC LIMIT 500";
    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    let logs = stmt
        .query_map([session_id], |row| {
            Ok(ExecutionLogEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                command: row.get(2)?,
                user: row.get(3)?,
                timestamp: row.get(4)?,
                result: row.get(5)?,
                error_message: row.get(6)?,
                return_code: row.get(7)?,
                requires_reboot: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();
    Ok(logs)
}

#[tauri::command]
pub fn get_system_status() -> CmdResult<SystemStatus> {
    let snap = crate::modules::monitor::snapshot(5);
    let windows_build = sysinfo::System::long_os_version().unwrap_or_else(|| std::env::consts::OS.to_string());
    Ok(SystemStatus {
        cpu_usage: snap.cpu_usage as f64,
        ram_usage: snap.ram_usage_percent as f64,
        windows_build,
        is_admin: crate::modules::monitor::is_elevated(),
    })
}

#[tauri::command]
pub fn list_top_processes(count: Option<usize>) -> CmdResult<Vec<crate::modules::monitor::ProcessInfo>> {
    Ok(crate::modules::monitor::snapshot(count.unwrap_or(8)).top_processes)
}

#[tauri::command]
pub fn list_detected_games(db: State<Db>) -> CmdResult<Vec<crate::modules::games::DetectedGame>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::modules::games::list_games(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn register_game(
    db: State<Db>,
    nome: String,
    executable_name: String,
    launcher: Option<String>,
    profile_id: Option<String>,
) -> CmdResult<String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::modules::games::register_game(&conn, &nome, &executable_name, launcher.as_deref(), profile_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_game_profile(db: State<Db>, game_id: String, profile_id: Option<String>) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::modules::games::set_game_profile(&conn, &game_id, profile_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_game(db: State<Db>, game_id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::modules::games::delete_game(&conn, &game_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_telemetry_comparisons(db: State<Db>) -> CmdResult<Vec<crate::engine::telemetry::TelemetryComparison>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::engine::telemetry::list_comparisons(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_telemetry_opt_in(db: State<Db>) -> CmdResult<bool> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let value: String = conn
        .query_row("SELECT value FROM user_settings WHERE key = 'telemetry_opt_in'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(value == "true")
}

#[tauri::command]
pub fn set_telemetry_opt_in(db: State<Db>, enabled: bool) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE user_settings SET value = ?1 WHERE key = 'telemetry_opt_in'",
        [if enabled { "true" } else { "false" }],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
