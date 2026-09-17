//! Allowlist fechada de ações que o Serviço privilegiado sabe executar.
//! `comando_de_aplicacao`/`comando_de_reversao` no catálogo do banco só têm
//! efeito se corresponderem a uma destas variantes.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionError {
    #[error("ação '{0}' não está na allowlist do aplicativo")]
    NotAllowed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionId {
    RegistrySetDword,
    RegistryRestoreValue,
    PowerSetScheme,
    PowerRestoreScheme,
    StartupDisableApp,
    StartupRestoreApp,
    ServiceSetStartType,
    ServiceRestoreStartType,
    ScheduledTaskDisable,
    ScheduledTaskRestore,
    NetworkSetDns,
    NetworkRestoreDns,
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
            "service_set_start_type" => Ok(Self::ServiceSetStartType),
            "service_restore_start_type" => Ok(Self::ServiceRestoreStartType),
            "scheduled_task_disable" => Ok(Self::ScheduledTaskDisable),
            "scheduled_task_restore" => Ok(Self::ScheduledTaskRestore),
            "network_set_dns" => Ok(Self::NetworkSetDns),
            "network_restore_dns" => Ok(Self::NetworkRestoreDns),
            other => Err(ActionError::NotAllowed(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegistrySetDword => "registry_set_dword",
            Self::RegistryRestoreValue => "registry_restore_value",
            Self::PowerSetScheme => "power_set_scheme",
            Self::PowerRestoreScheme => "power_restore_scheme",
            Self::StartupDisableApp => "startup_disable_app",
            Self::StartupRestoreApp => "startup_restore_app",
            Self::ServiceSetStartType => "service_set_start_type",
            Self::ServiceRestoreStartType => "service_restore_start_type",
            Self::ScheduledTaskDisable => "scheduled_task_disable",
            Self::ScheduledTaskRestore => "scheduled_task_restore",
            Self::NetworkSetDns => "network_set_dns",
            Self::NetworkRestoreDns => "network_restore_dns",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_action() {
        assert!(ActionId::parse("rm -rf /").is_err());
    }

    #[test]
    fn roundtrips_known_actions() {
        for action in [
            ActionId::RegistrySetDword,
            ActionId::ServiceSetStartType,
            ActionId::ScheduledTaskDisable,
            ActionId::NetworkSetDns,
        ] {
            assert_eq!(ActionId::parse(action.as_str()).unwrap(), action);
        }
    }
}
