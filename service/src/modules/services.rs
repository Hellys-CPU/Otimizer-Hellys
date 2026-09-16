//! Gerenciamento de serviços do Windows via Service Control Manager (SCM).
//!
//! ATENÇÃO: assim como os demais módulos `cfg(windows)`, esta implementação
//! não pôde ser compilada nem testada neste ambiente de desenvolvimento
//! (sem Windows disponível) — precisa de validação manual antes de uso em
//! produção. A allowlist de serviços protegidos (`PROTECTED_SERVICES` em
//! `shared::validation`) é a barreira testável e é aplicada antes de
//! qualquer chamada Win32.

use systemforge_shared::validation::{validate_service_name, ValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("operação de serviço não suportada nesta plataforma")]
    UnsupportedPlatform,
    #[error("erro do Win32 ao acessar o Service Control Manager: {0}")]
    Win32(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartType {
    Automatic,
    Manual,
    Disabled,
}

impl StartType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Automatic" => Some(Self::Automatic),
            "Manual" => Some(Self::Manual),
            "Disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::Manual => "Manual",
            Self::Disabled => "Disabled",
        }
    }
}

#[cfg(windows)]
mod win_impl {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::System::Services::{
        ChangeServiceConfigW, CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceConfigW,
        QUERY_SERVICE_CONFIGW, SC_MANAGER_CONNECT, SERVICE_AUTO_START, SERVICE_CHANGE_CONFIG,
        SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_NO_CHANGE, SERVICE_QUERY_CONFIG,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn to_win32(t: StartType) -> u32 {
        match t {
            StartType::Automatic => SERVICE_AUTO_START.0,
            StartType::Manual => SERVICE_DEMAND_START.0,
            StartType::Disabled => SERVICE_DISABLED.0,
        }
    }

    fn from_win32(v: u32) -> StartType {
        if v == SERVICE_AUTO_START.0 {
            StartType::Automatic
        } else if v == SERVICE_DISABLED.0 {
            StartType::Disabled
        } else {
            StartType::Manual
        }
    }

    pub fn query_start_type(service_name: &str) -> Result<StartType, ServiceError> {
        validate_service_name(service_name)?;
        unsafe {
            let sc_manager = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| ServiceError::Win32(e.to_string()))?;
            let name_w = wide(service_name);
            let service = match OpenServiceW(sc_manager, PCWSTR(name_w.as_ptr()), SERVICE_QUERY_CONFIG) {
                Ok(h) => h,
                Err(e) => {
                    let _ = CloseServiceHandle(sc_manager);
                    return Err(ServiceError::Win32(e.to_string()));
                }
            };

            let mut needed = 0u32;
            let _ = QueryServiceConfigW(service, None, 0, &mut needed);
            let mut buf = vec![0u8; needed as usize];
            let result = QueryServiceConfigW(
                service,
                Some(buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW),
                buf.len() as u32,
                &mut needed,
            );
            let _ = CloseServiceHandle(service);
            let _ = CloseServiceHandle(sc_manager);
            result.map_err(|e| ServiceError::Win32(e.to_string()))?;

            let config = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            Ok(from_win32(config.dwStartType.0))
        }
    }

    pub fn set_start_type(service_name: &str, start_type: StartType) -> Result<(), ServiceError> {
        validate_service_name(service_name)?;
        unsafe {
            let sc_manager = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)
                .map_err(|e| ServiceError::Win32(e.to_string()))?;
            let name_w = wide(service_name);
            let service = match OpenServiceW(sc_manager, PCWSTR(name_w.as_ptr()), SERVICE_CHANGE_CONFIG) {
                Ok(h) => h,
                Err(e) => {
                    let _ = CloseServiceHandle(sc_manager);
                    return Err(ServiceError::Win32(e.to_string()));
                }
            };

            let result = ChangeServiceConfigW(
                service,
                SERVICE_NO_CHANGE,
                windows::Win32::System::Services::SERVICE_START_TYPE(to_win32(start_type)),
                SERVICE_NO_CHANGE,
                PCWSTR::null(),
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            );
            let _ = CloseServiceHandle(service);
            let _ = CloseServiceHandle(sc_manager);
            result.map_err(|e| ServiceError::Win32(e.to_string()))
        }
    }
}

#[cfg(windows)]
pub use win_impl::{query_start_type, set_start_type};

#[cfg(not(windows))]
pub fn query_start_type(service_name: &str) -> Result<StartType, ServiceError> {
    validate_service_name(service_name)?;
    Err(ServiceError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn set_start_type(service_name: &str, _start_type: StartType) -> Result<(), ServiceError> {
    validate_service_name(service_name)?;
    Err(ServiceError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_protected_service_before_touching_win32() {
        let err = set_start_type("wuauserv", StartType::Disabled).unwrap_err();
        assert!(matches!(err, ServiceError::Validation(ValidationError::ProtectedService(_))));
    }

    #[test]
    fn start_type_round_trips_through_strings() {
        for t in [StartType::Automatic, StartType::Manual, StartType::Disabled] {
            assert_eq!(StartType::parse(t.as_str()), Some(t));
        }
    }
}
