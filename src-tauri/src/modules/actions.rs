//! Allowlist fechada de ações que o aplicativo é capaz de executar.
//!
//! `comando_de_aplicacao` / `comando_de_reversao` no catálogo do banco são
//! strings que só têm efeito se corresponderem a uma destas variantes — não
//! existe caminho para executar uma string arbitrária vinda do banco ou da UI.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("ação '{0}' não está na allowlist do aplicativo")]
    NotAllowed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionId {
    RegistrySetDword,
    RegistryRestoreValue,
    PowerSetScheme,
    PowerRestoreScheme,
    StartupDisableApp,
    StartupRestoreApp,
    RegistryDisableBackgroundApp,
    RegistryRestoreBackgroundApp,
}

impl ActionId {
    pub fn parse(s: &str) -> Result<Self, ActionError> {
        match s {
            "registry_set_dword" => Ok(Self::RegistrySetDword),
            "registry_restore_value" => Ok(Self::RegistryRestoreValue),
            "power_set_scheme" => Ok(Self::PowerSetScheme),
            "power_restore_scheme" => Ok(Self::PowerRestoreScheme),
            "startup_disable_app" => Ok(Self::StartupDisableApp),
            "startup_restore_app" => Ok(Self::StartupRestoreApp),
            "registry_disable_background_app" => Ok(Self::RegistryDisableBackgroundApp),
            "registry_restore_background_app" => Ok(Self::RegistryRestoreBackgroundApp),
            other => Err(ActionError::NotAllowed(other.to_string())),
        }
    }
}
