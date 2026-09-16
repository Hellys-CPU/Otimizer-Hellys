//! Leitura (só leitura) do Registro do Windows, usada para exibir o "valor
//! atual" no catálogo antes de aplicar um tweak.
//!
//! Qualquer ESCRITA/EXCLUSÃO de Registro passa exclusivamente pelo Serviço
//! privilegiado via `modules::service_client` — este módulo nunca grava.
//! Reaplica a mesma allowlist de `shared::validation` usada pelo Serviço.

use systemforge_shared::validation::ValidationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("operação de registro não suportada nesta plataforma (build fora do Windows)")]
    UnsupportedPlatform,
    #[error("erro do Win32 ao acessar o registro: {0}")]
    Win32(String),
}

#[cfg(windows)]
mod win_impl {
    use super::*;
    use systemforge_shared::validation::{validate_registry_path, validate_value_name};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
        KEY_READ,
    };

    fn root_hkey(hive: &str) -> HKEY {
        match hive {
            "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
            "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
            _ => unreachable!("validate_registry_path já filtrou a colmeia"),
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn read_dword(full_path: &str, value_name: &str) -> Result<Option<u32>, RegistryError> {
        let (hive, subkey) = validate_registry_path(full_path)?;
        validate_value_name(value_name)?;

        unsafe {
            let mut hkey = HKEY::default();
            let subkey_w = wide(subkey);
            let open = RegOpenKeyExW(root_hkey(hive), PCWSTR(subkey_w.as_ptr()), 0, KEY_READ, &mut hkey);
            if open.is_err() {
                return Ok(None);
            }

            let mut data: u32 = 0;
            let mut data_len = std::mem::size_of::<u32>() as u32;
            let name_w = wide(value_name);
            let result = RegQueryValueExW(
                hkey,
                PCWSTR(name_w.as_ptr()),
                None,
                None,
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_len),
            );
            let _ = RegCloseKey(hkey);

            if result.is_ok() {
                Ok(Some(data))
            } else if result == ERROR_FILE_NOT_FOUND {
                Ok(None)
            } else {
                Err(RegistryError::Win32(format!("{result:?}")))
            }
        }
    }
}

#[cfg(windows)]
pub use win_impl::read_dword;

#[cfg(not(windows))]
pub fn read_dword(full_path: &str, value_name: &str) -> Result<Option<u32>, RegistryError> {
    systemforge_shared::validation::validate_registry_path(full_path)?;
    systemforge_shared::validation::validate_value_name(value_name)?;
    Err(RegistryError::UnsupportedPlatform)
}
