//! Gerenciamento de aplicativos de inicialização via a chave `Run` do Registro
//! (HKCU/HKLM ...\Windows\CurrentVersion\Run). Usa o mesmo wrapper seguro de
//! `registry.rs` — nenhuma chamada direta ao Win32 a partir daqui.

#[cfg(windows)]
use super::registry::{validate_path, validate_value_name};
use super::registry::RegistryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error(transparent)]
    Registry(#[from] RegistryError),
}

pub const HKCU_RUN: &str = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";

/// Lê o valor REG_SZ de um item de inicialização (caminho do executável), se existir.
/// Implementação de leitura/escrita de REG_SZ arbitrário fica isolada aqui porque
/// só é necessária para este módulo (registry.rs hoje só expõe DWORD, usado pelos
/// 4 tweaks de Registro do catálogo inicial).
#[cfg(windows)]
pub fn read_run_value(location: &str, app_name: &str) -> Result<Option<String>, StartupError> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    };

    let (hive, subkey) = validate_path(location)?;
    validate_value_name(app_name)?;
    if hive != "HKEY_CURRENT_USER" {
        return Err(StartupError::Registry(RegistryError::HiveNotAllowed(hive.to_string())));
    }

    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };

    unsafe {
        let mut hkey = HKEY::default();
        let subkey_w = wide(subkey);
        if RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(subkey_w.as_ptr()), 0, KEY_READ, &mut hkey).is_err() {
            return Ok(None);
        }

        let name_w = wide(app_name);
        let mut buf = [0u16; 1024];
        let mut buf_len = (buf.len() * 2) as u32;
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(name_w.as_ptr()),
            None,
            None,
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_len),
        );
        let _ = RegCloseKey(hkey);

        match result {
            Ok(()) => {
                let len_u16 = (buf_len as usize) / 2;
                let s = String::from_utf16_lossy(&buf[..len_u16.saturating_sub(1)]);
                Ok(Some(s))
            }
            Err(_) => Ok(None),
        }
    }
}

#[cfg(windows)]
pub fn disable_startup_item(location: &str, app_name: &str) -> Result<Option<String>, StartupError> {
    use super::registry::delete_value;
    let previous = read_run_value(location, app_name)?;
    if previous.is_some() {
        delete_value(location, app_name)?;
    }
    Ok(previous)
}

#[cfg(windows)]
pub fn restore_startup_item(
    location: &str,
    app_name: &str,
    previous_value: &Option<String>,
) -> Result<(), StartupError> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE,
        REG_OPTION_NON_VOLATILE, REG_SZ,
    };

    let Some(value) = previous_value else {
        return Ok(()); // não existia antes — nada a restaurar
    };

    let (_, subkey) = validate_path(location)?;
    validate_value_name(app_name)?;
    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };

    unsafe {
        let mut hkey = HKEY::default();
        let subkey_w = wide(subkey);
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_w.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
        .map_err(|e| StartupError::Registry(RegistryError::Win32(e.to_string())))?;

        let name_w = wide(app_name);
        let value_w = wide(value);
        let bytes = std::slice::from_raw_parts(value_w.as_ptr() as *const u8, value_w.len() * 2);
        let result = RegSetValueExW(hkey, PCWSTR(name_w.as_ptr()), 0, REG_SZ, Some(bytes));
        let _ = RegCloseKey(hkey);
        result.map_err(|e| StartupError::Registry(RegistryError::Win32(e.to_string())))
    }
}

#[cfg(not(windows))]
pub fn read_run_value(_location: &str, _app_name: &str) -> Result<Option<String>, StartupError> {
    Err(StartupError::Registry(RegistryError::UnsupportedPlatform))
}

#[cfg(not(windows))]
pub fn disable_startup_item(_location: &str, _app_name: &str) -> Result<Option<String>, StartupError> {
    Err(StartupError::Registry(RegistryError::UnsupportedPlatform))
}

#[cfg(not(windows))]
pub fn restore_startup_item(
    _location: &str,
    _app_name: &str,
    _previous_value: &Option<String>,
) -> Result<(), StartupError> {
    Err(StartupError::Registry(RegistryError::UnsupportedPlatform))
}
