//! Único lugar do sistema que efetivamente grava no Registro do Windows.
//! Reaplica a mesma validação de `shared::validation` antes de qualquer
//! chamada Win32 — o Serviço nunca confia que o Core já validou.

use systemforge_shared::validation::{validate_registry_path, validate_value_name, ValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("operação de registro não suportada nesta plataforma")]
    UnsupportedPlatform,
    #[error("erro do Win32 ao acessar o registro: {0}")]
    Win32(String),
}

#[cfg(windows)]
mod win_impl {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE,
        REG_DWORD, REG_OPTION_NON_VOLATILE,
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
            if RegOpenKeyExW(root_hkey(hive), PCWSTR(subkey_w.as_ptr()), 0, KEY_READ, &mut hkey).is_err() {
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

            match result {
                Ok(()) => Ok(Some(data)),
                Err(e) if e.code().0 as u32 == ERROR_FILE_NOT_FOUND.0 => Ok(None),
                Err(e) => Err(RegistryError::Win32(e.to_string())),
            }
        }
    }

    pub fn write_dword(full_path: &str, value_name: &str, value: u32) -> Result<(), RegistryError> {
        let (hive, subkey) = validate_registry_path(full_path)?;
        validate_value_name(value_name)?;

        unsafe {
            let mut hkey = HKEY::default();
            let subkey_w = wide(subkey);
            RegCreateKeyExW(
                root_hkey(hive),
                PCWSTR(subkey_w.as_ptr()),
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut hkey,
                None,
            )
            .map_err(|e| RegistryError::Win32(e.to_string()))?;

            let name_w = wide(value_name);
            let bytes = value.to_le_bytes();
            let result = RegSetValueExW(hkey, PCWSTR(name_w.as_ptr()), 0, REG_DWORD, Some(&bytes));
            let _ = RegCloseKey(hkey);
            result.map_err(|e| RegistryError::Win32(e.to_string()))
        }
    }

    pub fn delete_value(full_path: &str, value_name: &str) -> Result<(), RegistryError> {
        let (hive, subkey) = validate_registry_path(full_path)?;
        validate_value_name(value_name)?;

        unsafe {
            let mut hkey = HKEY::default();
            let subkey_w = wide(subkey);
            if RegOpenKeyExW(root_hkey(hive), PCWSTR(subkey_w.as_ptr()), 0, KEY_WRITE, &mut hkey).is_err() {
                return Ok(());
            }
            let name_w = wide(value_name);
            let result = RegDeleteValueW(hkey, PCWSTR(name_w.as_ptr()));
            let _ = RegCloseKey(hkey);
            match result {
                Ok(()) => Ok(()),
                Err(e) if e.code().0 as u32 == ERROR_FILE_NOT_FOUND.0 => Ok(()),
                Err(e) => Err(RegistryError::Win32(e.to_string())),
            }
        }
    }
}

#[cfg(windows)]
pub use win_impl::{delete_value, read_dword, write_dword};

#[cfg(not(windows))]
pub fn read_dword(full_path: &str, value_name: &str) -> Result<Option<u32>, RegistryError> {
    validate_registry_path(full_path)?;
    validate_value_name(value_name)?;
    Err(RegistryError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn write_dword(full_path: &str, value_name: &str, _value: u32) -> Result<(), RegistryError> {
    validate_registry_path(full_path)?;
    validate_value_name(value_name)?;
    Err(RegistryError::UnsupportedPlatform)
}

#[cfg(not(windows))]
pub fn delete_value(full_path: &str, value_name: &str) -> Result<(), RegistryError> {
    validate_registry_path(full_path)?;
    validate_value_name(value_name)?;
    Err(RegistryError::UnsupportedPlatform)
}
