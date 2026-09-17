//! Configuração de DNS por adaptador via `netsh.exe` (mecanismo oficial),
//! argv fixo. Sempre lê o valor original antes de alterar, para permitir
//! restauração.

use std::process::Command;
use systemforge_shared::validation::{validate_network_token, ValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("falha ao executar netsh: {0}")]
    CommandFailed(String),
    #[error("netsh retornou código de erro: {0}")]
    NonZeroExit(i32),
}

/// Retorna os servidores DNS configurados no adaptador, na ordem em que o
/// Windows os lista. Lista vazia = configurado para obter DNS automaticamente
/// (DHCP).
pub fn get_dns(adapter_name: &str) -> Result<Vec<String>, NetworkError> {
    validate_network_token(adapter_name)?;

    let output = Command::new("netsh")
        .args(["interface", "ip", "show", "dns", &format!("name=\"{adapter_name}\"")])
        .output()
        .map_err(|e| NetworkError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(NetworkError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let servers = stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .split_whitespace()
                .find(|tok| tok.chars().filter(|c| *c == '.').count() == 3 && tok.chars().all(|c| c.is_ascii_digit() || c == '.'))
                .map(String::from)
        })
        .collect();
    Ok(servers)
}

pub fn set_dns_static(adapter_name: &str, primary: &str, secondary: Option<&str>) -> Result<(), NetworkError> {
    validate_network_token(adapter_name)?;
    validate_network_token(primary)?;
    if let Some(s) = secondary {
        validate_network_token(s)?;
    }

    let output = Command::new("netsh")
        .args([
            "interface",
            "ip",
            "set",
            "dns",
            &format!("name=\"{adapter_name}\""),
            "static",
            primary,
            "primary",
        ])
        .output()
        .map_err(|e| NetworkError::CommandFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(NetworkError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }

    if let Some(secondary) = secondary {
        let output = Command::new("netsh")
            .args([
                "interface",
                "ip",
                "add",
                "dns",
                &format!("name=\"{adapter_name}\""),
                &format!("addr={secondary}"),
                "index=2",
            ])
            .output()
            .map_err(|e| NetworkError::CommandFailed(e.to_string()))?;
        if !output.status.success() {
            return Err(NetworkError::NonZeroExit(output.status.code().unwrap_or(-1)));
        }
    }

    Ok(())
}

/// Restaura DNS automático (DHCP) — usado quando o valor original era "sem
/// servidores estáticos configurados".
pub fn set_dns_dhcp(adapter_name: &str) -> Result<(), NetworkError> {
    validate_network_token(adapter_name)?;

    let output = Command::new("netsh")
        .args(["interface", "ip", "set", "dns", &format!("name=\"{adapter_name}\""), "dhcp"])
        .output()
        .map_err(|e| NetworkError::CommandFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(NetworkError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_adapter_name_with_shell_metacharacters() {
        assert!(matches!(get_dns("Wi-Fi\" & calc.exe"), Err(NetworkError::Validation(_))));
    }

    #[test]
    fn rejects_dns_value_with_quotes() {
        assert!(matches!(
            set_dns_static("Ethernet", "8.8.8.8\"", None),
            Err(NetworkError::Validation(_))
        ));
    }
}
