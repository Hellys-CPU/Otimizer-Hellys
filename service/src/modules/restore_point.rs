//! Ponto de restauração do Windows via `Checkpoint-Computer` (PowerShell,
//! módulo built-in `Microsoft.PowerShell.Management`).
//!
//! A descrição é passada por variável de ambiente, não interpolada no texto
//! do script — evita qualquer necessidade de escapar aspas/`;`/backticks
//! vindos do Core.

use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RestorePointError {
    #[error("falha ao executar powershell: {0}")]
    CommandFailed(String),
    #[error("não foi possível criar o ponto de restauração: {0}")]
    Failed(String),
}

const SCRIPT: &str = "Checkpoint-Computer -Description $env:SFO_RESTORE_DESC -RestorePointType MODIFY_SETTINGS";

pub fn create(description: &str) -> Result<(), RestorePointError> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .env("SFO_RESTORE_DESC", description)
        .output()
        .map_err(|e| RestorePointError::CommandFailed(e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = if stderr.trim().is_empty() {
            format!("código de saída {}", output.status.code().unwrap_or(-1))
        } else {
            stderr.trim().to_string()
        };
        Err(RestorePointError::Failed(message))
    }
}
