//! Integração com planos de energia via `powercfg.exe` (mecanismo oficial do Windows).
//!
//! Nunca construímos uma string de shell: `Command::new` recebe o binário fixo e
//! cada argumento é passado separadamente no vetor de argv, então não há
//! interpretação de shell nem espaço para injeção via GUID.

use thiserror::Error;
use std::process::Command;

#[derive(Debug, Error)]
pub enum PowerError {
    #[error("GUID de esquema de energia não é um GUID válido")]
    InvalidGuid,
    #[error("falha ao executar powercfg: {0}")]
    CommandFailed(String),
    #[error("powercfg retornou código de erro: {0}")]
    NonZeroExit(i32),
}

/// Esquemas embutidos do Windows (GUIDs bem conhecidos e documentados pela Microsoft).
pub const SCHEME_HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const SCHEME_BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const SCHEME_POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";

fn validate_guid(guid: &str) -> Result<(), PowerError> {
    let is_hex_or_dash = guid.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    if guid.len() != 36 || !is_hex_or_dash {
        return Err(PowerError::InvalidGuid);
    }
    Ok(())
}

pub fn get_active_scheme() -> Result<String, PowerError> {
    let output = Command::new("powercfg")
        .args(["/getactivescheme"])
        .output()
        .map_err(|e| PowerError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(PowerError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Formato: "Esquema de energia atual do GUID: <guid>  (<nome>)"
    stdout
        .split_whitespace()
        .find(|tok| tok.len() == 36 && tok.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
        .map(String::from)
        .ok_or(PowerError::InvalidGuid)
}

pub fn set_active_scheme(scheme_guid: &str) -> Result<(), PowerError> {
    validate_guid(scheme_guid)?;

    let output = Command::new("powercfg")
        .args(["/setactive", scheme_guid])
        .output()
        .map_err(|e| PowerError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(PowerError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }
    Ok(())
}
