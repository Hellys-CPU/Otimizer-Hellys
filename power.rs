//! Integração com planos de energia via `powercfg.exe` (mecanismo oficial).
//! Argv fixo — nunca monta uma string de shell.

use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PowerError {
    #[error("GUID de esquema de energia não é um GUID válido")]
    InvalidGuid,
    #[error("falha ao executar powercfg: {0}")]
    CommandFailed(String),
    #[error("powercfg retornou código de erro: {0}")]
    NonZeroExit(i32),
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_guid() {
        assert!(matches!(set_active_scheme("'; rm -rf /"), Err(PowerError::InvalidGuid)));
    }

    #[test]
    fn accepts_known_scheme_guid_shape() {
        assert!(validate_guid(systemforge_shared::SCHEME_HIGH_PERFORMANCE).is_ok());
    }
}
