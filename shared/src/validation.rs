//! Validação de entrada compartilhada pelo Core e pelo Serviço — a mesma
//! allowlist e as mesmas regras de sanitização são aplicadas nos dois lados
//! (o Core valida antes de pedir, o Serviço valida de novo antes de executar;
//! nunca confia cegamente no que chega pelo pipe).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationError {
    #[error("colmeia de registro não permitida: {0}")]
    HiveNotAllowed(String),
    #[error("caminho de registro inválido: {0}")]
    InvalidPath(String),
    #[error("nome de valor inválido: {0}")]
    InvalidValueName(String),
    #[error("nome de serviço inválido: {0}")]
    InvalidServiceName(String),
    #[error("caminho de tarefa agendada inválido: {0}")]
    InvalidTaskPath(String),
    #[error("serviço '{0}' está na lista de serviços protegidos e não pode ser alterado")]
    ProtectedService(String),
    #[error("endereço/adaptador de rede inválido: {0}")]
    InvalidNetworkInput(String),
}

pub const ALLOWED_HIVES: &[&str] = &["HKEY_CURRENT_USER", "HKEY_LOCAL_MACHINE"];

/// Serviços que o aplicativo NUNCA altera, mesmo que o usuário peça — proteção
/// de segurança/rede essencial e Windows Update (requisito explícito do produto).
pub const PROTECTED_SERVICES: &[&str] = &[
    "wuauserv",     // Windows Update
    "WinDefend",    // Microsoft Defender Antivirus
    "mpssvc",       // Windows Defender Firewall
    "SecurityHealthService",
    "wscsvc",       // Security Center
    "BFE",          // Base Filtering Engine (firewall)
    "Dnscache",     // DNS Client
    "Dhcp",
    "nsi",          // Network Store Interface
    "RpcSs",
    "LSM",
    "EventLog",
    "Winmgmt", // WMI
];

pub fn validate_registry_path(full_path: &str) -> Result<(&str, &str), ValidationError> {
    let (hive, subkey) = full_path
        .split_once('\\')
        .ok_or_else(|| ValidationError::InvalidPath(full_path.to_string()))?;

    if !ALLOWED_HIVES.contains(&hive) {
        return Err(ValidationError::HiveNotAllowed(hive.to_string()));
    }

    if subkey.is_empty() || subkey.contains("..") || subkey.chars().any(|c| c.is_control()) {
        return Err(ValidationError::InvalidPath(full_path.to_string()));
    }

    Ok((hive, subkey))
}

pub fn validate_value_name(name: &str) -> Result<(), ValidationError> {
    if name.chars().any(|c| c.is_control()) {
        return Err(ValidationError::InvalidValueName(name.to_string()));
    }
    Ok(())
}

pub fn validate_service_name(name: &str) -> Result<(), ValidationError> {
    let valid = !name.is_empty()
        && name.len() <= 256
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ' '));
    if !valid {
        return Err(ValidationError::InvalidServiceName(name.to_string()));
    }
    if PROTECTED_SERVICES.iter().any(|s| s.eq_ignore_ascii_case(name)) {
        return Err(ValidationError::ProtectedService(name.to_string()));
    }
    Ok(())
}

/// Caminho de tarefa agendada no formato usado pelo Task Scheduler, ex.:
/// `\Microsoft\Windows\AppID\PolicyConverter` — sempre começa com `\`.
pub fn validate_task_path(path: &str) -> Result<(), ValidationError> {
    let valid = path.starts_with('\\')
        && !path.contains("..")
        && path.chars().all(|c| !c.is_control());
    if !valid {
        return Err(ValidationError::InvalidTaskPath(path.to_string()));
    }
    Ok(())
}

/// Nome de adaptador de rede (ex.: "Wi-Fi", "Ethernet") ou endereço IPv4 de DNS.
pub fn validate_network_token(token: &str) -> Result<(), ValidationError> {
    let valid = !token.is_empty()
        && token.len() <= 128
        && token.chars().all(|c| !c.is_control() && c != '"' && c != '&' && c != '|');
    if !valid {
        return Err(ValidationError::InvalidNetworkInput(token.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_hive_not_allowed() {
        assert!(matches!(
            validate_registry_path("HKEY_CLASSES_ROOT\\Foo"),
            Err(ValidationError::HiveNotAllowed(_))
        ));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_registry_path("HKEY_CURRENT_USER\\Software\\..\\..\\Windows").is_err());
    }

    #[test]
    fn accepts_valid_registry_path() {
        let (hive, subkey) = validate_registry_path("HKEY_CURRENT_USER\\Software\\Microsoft\\GameBar").unwrap();
        assert_eq!(hive, "HKEY_CURRENT_USER");
        assert_eq!(subkey, "Software\\Microsoft\\GameBar");
    }

    #[test]
    fn rejects_protected_service() {
        assert!(matches!(
            validate_service_name("wuauserv"),
            Err(ValidationError::ProtectedService(_))
        ));
        assert!(matches!(
            validate_service_name("WinDefend"),
            Err(ValidationError::ProtectedService(_))
        ));
    }

    #[test]
    fn accepts_ordinary_service_name() {
        assert!(validate_service_name("SysMain").is_ok());
    }

    #[test]
    fn rejects_task_path_without_leading_backslash() {
        assert!(validate_task_path("Microsoft\\Windows\\Foo").is_err());
    }

    #[test]
    fn accepts_valid_task_path() {
        assert!(validate_task_path("\\Microsoft\\Windows\\AppID\\PolicyConverter").is_ok());
    }

    #[test]
    fn rejects_network_token_with_shell_metacharacters() {
        assert!(validate_network_token("8.8.8.8\" & calc.exe").is_err());
    }
}
