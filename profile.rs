use super::{backup, EngineError};
use crate::models::ExecutionSession;
use rusqlite::Connection;
use systemforge_shared::ActionId;
use uuid::Uuid;

struct OptimizationRow {
    id: String,
    comando_de_aplicacao: String,
    comando_de_reversao: String,
    caminho: Option<String>,
    nome_do_valor: Option<String>,
    valor_recomendado: Option<String>,
}

fn fetch_optimization(conn: &Connection, id: &str) -> Result<OptimizationRow, EngineError> {
    conn.query_row(
        "SELECT id, comando_de_aplicacao, comando_de_reversao, caminho, nome_do_valor, valor_recomendado
         FROM optimizations WHERE id = ?1",
        [id],
        |row| {
            Ok(OptimizationRow {
                id: row.get(0)?,
                comando_de_aplicacao: row.get(1)?,
                comando_de_reversao: row.get(2)?,
                caminho: row.get(3)?,
                nome_do_valor: row.get(4)?,
                valor_recomendado: row.get(5)?,
            })
        },
    )
    .map_err(|_| EngineError::OptimizationNotFound(id.to_string()))
}

fn dispatch_apply(conn: &Connection, session_id: &str, opt: &OptimizationRow) -> Result<(), EngineError> {
    let action = ActionId::parse(&opt.comando_de_aplicacao)?;
    match action {
        ActionId::RegistrySetDword => {
            let path = opt.caminho.as_deref().ok_or_else(|| {
                EngineError::RequiresManualSelection(opt.id.clone())
            })?;
            let value_name = opt.nome_do_valor.as_deref().ok_or_else(|| {
                EngineError::RequiresManualSelection(opt.id.clone())
            })?;
            let recommended: u32 = opt
                .valor_recomendado
                .as_deref()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| EngineError::RequiresManualSelection(opt.id.clone()))?;
            backup::apply_registry_dword(conn, session_id, &opt.id, path, value_name, recommended)
        }
        ActionId::PowerSetScheme => {
            let target_guid = opt
                .valor_recomendado
                .as_deref()
                .filter(|v| v.len() == 36)
                .unwrap_or(systemforge_shared::SCHEME_HIGH_PERFORMANCE);
            backup::apply_power_scheme(conn, session_id, &opt.id, target_guid)
        }
        ActionId::StartupDisableApp => {
            // Requer seleção manual do item pelo usuário (não há um único app
            // "correto" universal) — a UI deve chamar apply_optimization com o
            // item escolhido via um comando dedicado, não via aplicação de perfil.
            Err(EngineError::RequiresManualSelection(opt.id.clone()))
        }
        ActionId::ServiceSetStartType => {
            let service_name = opt.caminho.as_deref().ok_or_else(|| {
                EngineError::RequiresManualSelection(opt.id.clone())
            })?;
            let start_type = opt.valor_recomendado.as_deref().ok_or_else(|| {
                EngineError::RequiresManualSelection(opt.id.clone())
            })?;
            backup::apply_service_start_type(conn, session_id, &opt.id, service_name, start_type)
        }
        ActionId::ScheduledTaskDisable => {
            let task_path = opt.caminho.as_deref().ok_or_else(|| {
                EngineError::RequiresManualSelection(opt.id.clone())
            })?;
            backup::apply_scheduled_task_disable(conn, session_id, &opt.id, task_path)
        }
        // Rede exige seleção do adaptador pelo usuário — sem um único
        // "adaptador correto" universal, não dá para aplicar via perfil.
        ActionId::ServiceRestoreStartType
        | ActionId::ScheduledTaskRestore
        | ActionId::NetworkSetDns
        | ActionId::NetworkRestoreDns
        | ActionId::RegistryRestoreValue
        | ActionId::PowerRestoreScheme
        | ActionId::StartupRestoreApp => Err(EngineError::RequiresManualSelection(opt.id.clone())),
    }
}

fn dispatch_restore(conn: &Connection, session_id: &str, opt: &OptimizationRow) -> Result<(), EngineError> {
    let action = ActionId::parse(&opt.comando_de_reversao)?;
    match action {
        ActionId::RegistryRestoreValue => backup::restore_registry_dword(conn, session_id, &opt.id),
        ActionId::PowerRestoreScheme => backup::restore_power_scheme(conn, session_id, &opt.id),
        ActionId::StartupRestoreApp => backup::restore_startup_item(conn, session_id, &opt.id),
        ActionId::ServiceRestoreStartType => backup::restore_service_start_type(conn, session_id, &opt.id),
        ActionId::ScheduledTaskRestore => backup::restore_scheduled_task(conn, session_id, &opt.id),
        _ => Err(EngineError::NoSnapshotToRestore(opt.id.clone())),
    }
}

pub fn apply_profile(conn: &Connection, profile_id: &str) -> Result<ExecutionSession, EngineError> {
    let exists: bool = conn
        .query_row("SELECT EXISTS(SELECT 1 FROM profiles WHERE id = ?1)", [profile_id], |r| r.get(0))?;
    if !exists {
        return Err(EngineError::ProfileNotFound(profile_id.to_string()));
    }

    let session_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO execution_sessions (id, profile_id, status) VALUES (?1, ?2, 'pending')",
        rusqlite::params![session_id, profile_id],
    )?;

    let mut stmt = conn.prepare(
        "SELECT po.optimization_id FROM profile_optimizations po JOIN optimizations o ON o.id = po.optimization_id WHERE po.profile_id = ?1 AND po.proibido = 0 AND o.risco != 'experimental'",
    )?;
    let optimization_ids: Vec<String> = stmt
        .query_map([profile_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    let mut any_failure = false;
    for opt_id in &optimization_ids {
        let opt = fetch_optimization(conn, opt_id)?;
        if let Err(e) = dispatch_apply(conn, &session_id, &opt) {
            // Falha (ou necessidade de seleção manual) em UM tweak não aborta o
            // perfil inteiro — cada tweak é independente e já foi logado.
            let _ = e;
            any_failure = true;
        }
    }

    let final_status = if any_failure { "partial" } else { "applied" };
    conn.execute(
        "UPDATE execution_sessions SET status = ?1, finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
        rusqlite::params![final_status, session_id],
    )?;
    conn.execute(
        "UPDATE profiles SET ativo = 0",
        [],
    )?;
    conn.execute(
        "UPDATE profiles SET ativo = 1, ultima_aplicacao = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [profile_id],
    )?;

    conn.query_row(
        "SELECT id, profile_id, started_at, finished_at, status FROM execution_sessions WHERE id = ?1",
        [&session_id],
        |row| {
            Ok(ExecutionSession {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                status: row.get(4)?,
            })
        },
    )
    .map_err(EngineError::Db)
}

pub fn restore_profile(conn: &Connection, profile_id: &str) -> Result<(), EngineError> {
    let session_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO execution_sessions (id, profile_id, status) VALUES (?1, ?2, 'pending')",
        rusqlite::params![session_id, profile_id],
    )?;

    let mut stmt = conn.prepare(
        "SELECT po.optimization_id FROM profile_optimizations po JOIN optimizations o ON o.id = po.optimization_id WHERE po.profile_id = ?1 AND po.proibido = 0 AND o.risco != 'experimental'",
    )?;
    let optimization_ids: Vec<String> = stmt
        .query_map([profile_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    for opt_id in &optimization_ids {
        let opt = fetch_optimization(conn, opt_id)?;
        let _ = dispatch_restore(conn, &session_id, &opt);
    }

    conn.execute(
        "UPDATE execution_sessions SET status = 'restored', finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [&session_id],
    )?;
    conn.execute("UPDATE profiles SET ativo = 0 WHERE id = ?1", [profile_id])?;
    Ok(())
}

pub fn restore_session(conn: &Connection, session_id: &str) -> Result<(), EngineError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT optimization_id FROM optimization_snapshots WHERE session_id = ?1 AND status = 'applied'",
    )?;
    let optimization_ids: Vec<String> = stmt
        .query_map([session_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    for opt_id in &optimization_ids {
        let opt = fetch_optimization(conn, opt_id)?;
        let _ = dispatch_restore(conn, session_id, &opt);
    }

    conn.execute(
        "UPDATE execution_sessions SET status = 'restored', finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [session_id],
    )?;
    Ok(())
}

pub fn apply_single_optimization(conn: &Connection, optimization_id: &str) -> Result<(), EngineError> {
    let session_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO execution_sessions (id, profile_id, status) VALUES (?1, NULL, 'pending')",
        [&session_id],
    )?;
    let opt = fetch_optimization(conn, optimization_id)?;
    let result = dispatch_apply(conn, &session_id, &opt);
    let status = if result.is_ok() { "applied" } else { "failed" };
    conn.execute(
        "UPDATE execution_sessions SET status = ?1, finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
        rusqlite::params![status, session_id],
    )?;
    result
}

pub fn restore_single_optimization(conn: &Connection, optimization_id: &str) -> Result<(), EngineError> {
    let session_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO execution_sessions (id, profile_id, status) VALUES (?1, NULL, 'pending')",
        [&session_id],
    )?;
    let opt = fetch_optimization(conn, optimization_id)?;
    let result = dispatch_restore(conn, &session_id, &opt);
    let status = if result.is_ok() { "restored" } else { "failed" };
    conn.execute(
        "UPDATE execution_sessions SET status = ?1, finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
        rusqlite::params![status, session_id],
    )?;
    result
}
