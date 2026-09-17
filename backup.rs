//! Motor de backup e restauração.
//!
//! Regra central: NUNCA aplicar uma alteração sem antes gravar, na mesma
//! transação lógica, o valor original em `optimization_snapshots` + a tabela
//! específica do tipo de tweak. Restaurar sempre lê o snapshot mais recente
//! ainda `applied` daquela otimização.
//!
//! Toda escrita efetiva (Registro/Energia/Startup/Serviços/Tarefas/Rede)
//! passa por `modules::service_client`, que fala com o Serviço privilegiado
//! — este arquivo nunca chama Win32 diretamente.

use super::EngineError;
use crate::modules::{registry, service_client};
use rusqlite::Connection;
use uuid::Uuid;

fn log(
    conn: &Connection,
    session_id: &str,
    command: &str,
    result: &str,
    error_message: Option<&str>,
    return_code: Option<i64>,
    requires_reboot: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO execution_logs (id, session_id, command, user_name, result, error_message, return_code, requires_reboot)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            session_id,
            command,
            std::env::var("USERNAME").unwrap_or_else(|_| "unknown".into()),
            result,
            error_message,
            return_code,
            requires_reboot as i64,
        ],
    )?;
    Ok(())
}

fn new_snapshot(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    valor_original: Option<&str>,
    valor_aplicado: Option<&str>,
) -> rusqlite::Result<String> {
    let snapshot_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO optimization_snapshots (id, session_id, optimization_id, valor_original, valor_aplicado)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![snapshot_id, session_id, optimization_id, valor_original, valor_aplicado],
    )?;
    Ok(snapshot_id)
}

/// Aplica um tweak de Registro do tipo DWORD, salvando o valor original antes de escrever.
pub fn apply_registry_dword(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    path: &str,
    value_name: &str,
    recommended: u32,
) -> Result<(), EngineError> {
    // Leitura do valor original ainda é feita localmente (HKCU/HKLM leitura
    // não exige elevação) — só a escrita passa pelo Serviço.
    let original = registry::read_dword(path, value_name).unwrap_or(None);

    let apply_result = service_client::registry_write_dword(path, value_name, recommended);
    let command_desc = format!("registry_set_dword {path}\\{value_name} = {recommended}");

    match &apply_result {
        Ok(()) => {
            let snapshot_id = new_snapshot(
                conn,
                session_id,
                optimization_id,
                original.map(|v| v.to_string()).as_deref(),
                Some(&recommended.to_string()),
            )?;
            conn.execute(
                "INSERT INTO registry_backups (id, snapshot_id, caminho, nome_do_valor, tipo_do_valor, valor_original_raw, existia)
                 VALUES (?1, ?2, ?3, ?4, 'REG_DWORD', ?5, ?6)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    snapshot_id,
                    path,
                    value_name,
                    original.map(|v| v.to_string()),
                    original.is_some() as i64,
                ],
            )?;
            conn.execute(
                "UPDATE optimizations SET valor_atual = ?1 WHERE id = ?2",
                rusqlite::params![recommended.to_string(), optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

/// Restaura o valor de Registro mais recente ainda aplicado para esta otimização.
pub fn restore_registry_dword(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
) -> Result<(), EngineError> {
    let row: Option<(String, String, String, Option<String>, i64)> = conn
        .query_row(
            "SELECT rb.id, rb.caminho, rb.nome_do_valor, rb.valor_original_raw, rb.existia
             FROM registry_backups rb
             JOIN optimization_snapshots os ON os.id = rb.snapshot_id
             WHERE os.optimization_id = ?1 AND os.status = 'applied'
             ORDER BY os.aplicado_em DESC LIMIT 1",
            [optimization_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .ok();

    let Some((_backup_id, path, value_name, original_raw, existia)) = row else {
        return Err(EngineError::NoSnapshotToRestore(optimization_id.to_string()));
    };

    let command_desc = format!("registry_restore_value {path}\\{value_name}");
    let result = if existia == 1 {
        let value: u32 = original_raw.clone().unwrap_or_default().parse().unwrap_or(0);
        service_client::registry_write_dword(&path, &value_name, value)
    } else {
        service_client::registry_delete_value(&path, &value_name)
    };

    match result {
        Ok(()) => {
            conn.execute(
                "UPDATE optimization_snapshots SET status = 'restored', restaurado_em = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE optimization_id = ?1 AND status = 'applied'",
                [optimization_id],
            )?;
            conn.execute(
                "UPDATE optimizations SET valor_atual = ?1 WHERE id = ?2",
                rusqlite::params![original_raw, optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

pub fn apply_power_scheme(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    target_guid: &str,
) -> Result<(), EngineError> {
    let original = service_client::power_get_active_scheme().ok().flatten();
    let command_desc = format!("power_set_scheme {target_guid}");

    match service_client::power_set_active_scheme(target_guid) {
        Ok(()) => {
            let snapshot_id = new_snapshot(
                conn,
                session_id,
                optimization_id,
                original.as_deref(),
                Some(target_guid),
            )?;
            conn.execute(
                "INSERT INTO power_plan_snapshots (id, snapshot_id, active_scheme_guid) VALUES (?1, ?2, ?3)",
                rusqlite::params![Uuid::new_v4().to_string(), snapshot_id, original.unwrap_or_default()],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

pub fn restore_power_scheme(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
) -> Result<(), EngineError> {
    let original_guid: Option<String> = conn
        .query_row(
            "SELECT pps.active_scheme_guid
             FROM power_plan_snapshots pps
             JOIN optimization_snapshots os ON os.id = pps.snapshot_id
             WHERE os.optimization_id = ?1 AND os.status = 'applied'
             ORDER BY os.aplicado_em DESC LIMIT 1",
            [optimization_id],
            |row| row.get(0),
        )
        .ok();

    let Some(guid) = original_guid.filter(|g| !g.is_empty()) else {
        return Err(EngineError::NoSnapshotToRestore(optimization_id.to_string()));
    };

    let command_desc = format!("power_restore_scheme {guid}");
    match service_client::power_set_active_scheme(&guid) {
        Ok(()) => {
            conn.execute(
                "UPDATE optimization_snapshots SET status = 'restored', restaurado_em = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE optimization_id = ?1 AND status = 'applied'",
                [optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

pub fn apply_startup_disable(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    location: &str,
    app_name: &str,
) -> Result<(), EngineError> {
    let command_desc = format!("startup_disable_app {location}\\{app_name}");

    match service_client::startup_disable(location, app_name) {
        Ok(previous) => {
            let snapshot_id = new_snapshot(conn, session_id, optimization_id, previous.as_deref(), None)?;
            conn.execute(
                "INSERT INTO startup_snapshots (id, snapshot_id, app_name, executable_path, was_enabled, location)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    snapshot_id,
                    app_name,
                    previous,
                    previous.is_some() as i64,
                    location,
                ],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

pub fn restore_startup_item(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
) -> Result<(), EngineError> {
    let row: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT ss.location, ss.app_name, ss.executable_path
             FROM startup_snapshots ss
             JOIN optimization_snapshots os ON os.id = ss.snapshot_id
             WHERE os.optimization_id = ?1 AND os.status = 'applied'
             ORDER BY os.aplicado_em DESC LIMIT 1",
            [optimization_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let Some((location, app_name, executable_path)) = row else {
        return Err(EngineError::NoSnapshotToRestore(optimization_id.to_string()));
    };

    let command_desc = format!("startup_restore_app {location}\\{app_name}");
    match service_client::startup_restore(&location, &app_name, executable_path) {
        Ok(()) => {
            conn.execute(
                "UPDATE optimization_snapshots SET status = 'restored', restaurado_em = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE optimization_id = ?1 AND status = 'applied'",
                [optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

/// Muda o tipo de inicialização de um serviço do Windows, salvando o original.
pub fn apply_service_start_type(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    service_name: &str,
    target_start_type: &str,
) -> Result<(), EngineError> {
    let original = service_client::service_query_start_type(service_name).ok().flatten();
    let command_desc = format!("service_set_start_type {service_name} = {target_start_type}");

    match service_client::service_set_start_type(service_name, target_start_type) {
        Ok(()) => {
            let snapshot_id =
                new_snapshot(conn, session_id, optimization_id, original.as_deref(), Some(target_start_type))?;
            conn.execute(
                "INSERT INTO service_snapshots (id, snapshot_id, service_name, start_type_original, status_original, dependencies_json)
                 VALUES (?1, ?2, ?3, ?4, NULL, '[]')",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    snapshot_id,
                    service_name,
                    original.clone().unwrap_or_default(),
                ],
            )?;
            conn.execute(
                "UPDATE optimizations SET valor_atual = ?1 WHERE id = ?2",
                rusqlite::params![target_start_type, optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

/// Restaura o tipo de inicialização original do serviço.
pub fn restore_service_start_type(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
) -> Result<(), EngineError> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT sv.service_name, sv.start_type_original
             FROM service_snapshots sv
             JOIN optimization_snapshots os ON os.id = sv.snapshot_id
             WHERE os.optimization_id = ?1 AND os.status = 'applied'
             ORDER BY os.aplicado_em DESC LIMIT 1",
            [optimization_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let Some((service_name, original_start_type)) = row.filter(|(_, t)| !t.is_empty()) else {
        return Err(EngineError::NoSnapshotToRestore(optimization_id.to_string()));
    };

    let command_desc = format!("service_restore_start_type {service_name} -> {original_start_type}");
    match service_client::service_set_start_type(&service_name, &original_start_type) {
        Ok(()) => {
            conn.execute(
                "UPDATE optimization_snapshots SET status = 'restored', restaurado_em = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE optimization_id = ?1 AND status = 'applied'",
                [optimization_id],
            )?;
            conn.execute(
                "UPDATE optimizations SET valor_atual = ?1 WHERE id = ?2",
                rusqlite::params![original_start_type, optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

/// Desativa uma tarefa agendada, salvando se ela estava ativa antes.
pub fn apply_scheduled_task_disable(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
    task_path: &str,
) -> Result<(), EngineError> {
    let was_enabled = service_client::scheduled_task_query_enabled(task_path).unwrap_or(true);
    let command_desc = format!("scheduled_task_disable {task_path}");

    match service_client::scheduled_task_set_enabled(task_path, false) {
        Ok(()) => {
            let snapshot_id = new_snapshot(
                conn,
                session_id,
                optimization_id,
                Some(&was_enabled.to_string()),
                Some("false"),
            )?;
            conn.execute(
                "INSERT INTO scheduled_task_snapshots (id, snapshot_id, task_path, was_enabled, task_xml_backup)
                 VALUES (?1, ?2, ?3, ?4, '')",
                rusqlite::params![Uuid::new_v4().to_string(), snapshot_id, task_path, was_enabled as i64],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}

/// Restaura o estado ativo/inativo original da tarefa agendada.
pub fn restore_scheduled_task(
    conn: &Connection,
    session_id: &str,
    optimization_id: &str,
) -> Result<(), EngineError> {
    let row: Option<(String, i64)> = conn
        .query_row(
            "SELECT sts.task_path, sts.was_enabled
             FROM scheduled_task_snapshots sts
             JOIN optimization_snapshots os ON os.id = sts.snapshot_id
             WHERE os.optimization_id = ?1 AND os.status = 'applied'
             ORDER BY os.aplicado_em DESC LIMIT 1",
            [optimization_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let Some((task_path, was_enabled)) = row else {
        return Err(EngineError::NoSnapshotToRestore(optimization_id.to_string()));
    };

    let command_desc = format!("scheduled_task_restore {task_path}");
    match service_client::scheduled_task_set_enabled(&task_path, was_enabled != 0) {
        Ok(()) => {
            conn.execute(
                "UPDATE optimization_snapshots SET status = 'restored', restaurado_em = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE optimization_id = ?1 AND status = 'applied'",
                [optimization_id],
            )?;
            log(conn, session_id, &command_desc, "success", None, Some(0), false)?;
            Ok(())
        }
        Err(e) => {
            log(conn, session_id, &command_desc, "error", Some(&e.to_string()), None, false)?;
            Err(EngineError::Service(e.to_string()))
        }
    }
}
