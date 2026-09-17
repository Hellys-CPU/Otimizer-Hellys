//! Tarefas agendadas via `schtasks.exe` (mecanismo oficial), com argv fixo —
//! evita reimplementar a COM `ITaskService`, que tem superfície de erro
//! muito maior e não seria testável aqui de qualquer forma.

use std::process::Command;
use systemforge_shared::validation::{validate_task_path, ValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScheduledTaskError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("falha ao executar schtasks: {0}")]
    CommandFailed(String),
    #[error("schtasks retornou código de erro: {0}")]
    NonZeroExit(i32),
    #[error("não foi possível determinar o estado da tarefa a partir da saída do schtasks")]
    UnparseableOutput,
}

pub fn query_enabled(task_path: &str) -> Result<bool, ScheduledTaskError> {
    validate_task_path(task_path)?;

    let output = Command::new("schtasks")
        .args(["/Query", "/TN", task_path, "/FO", "LIST", "/V"])
        .output()
        .map_err(|e| ScheduledTaskError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(ScheduledTaskError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("Scheduled Task State") {
                return Ok(value.trim().eq_ignore_ascii_case("Enabled"));
            }
        }
    }
    Err(ScheduledTaskError::UnparseableOutput)
}

pub fn set_enabled(task_path: &str, enabled: bool) -> Result<(), ScheduledTaskError> {
    validate_task_path(task_path)?;

    let flag = if enabled { "/ENABLE" } else { "/DISABLE" };
    let output = Command::new("schtasks")
        .args(["/Change", "/TN", task_path, flag])
        .output()
        .map_err(|e| ScheduledTaskError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(ScheduledTaskError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_task_path_without_leading_backslash() {
        assert!(matches!(query_enabled("Microsoft\\Windows\\Foo"), Err(ScheduledTaskError::Validation(_))));
    }

    #[test]
    fn rejects_task_path_with_traversal() {
        assert!(matches!(
            set_enabled("\\Microsoft\\..\\Windows", false),
            Err(ScheduledTaskError::Validation(_))
        ));
    }
}
