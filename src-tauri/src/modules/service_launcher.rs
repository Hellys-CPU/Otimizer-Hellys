//! Auto-start do Serviço privilegiado via UAC quando o Core detecta que ele
//! não está rodando. Isso existe porque antes disso o usuário precisava abrir
//! um segundo terminal como Administrador manualmente — qualquer clique em
//! "Aplicar" sem isso feito falhava com um erro de token ausente que parecia
//! bug, mas era só o Serviço nunca ter sido iniciado.
//!
//! Dispara a elevação no máximo uma vez por sessão do Core: se o usuário
//! negar o UAC, não fica reabrindo o prompt a cada clique.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

static LAUNCH_ATTEMPTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("não foi possível localizar systemforge-service.exe ao lado do executável do Core")]
    ExecutableNotFound,
    #[error("a elevação (UAC) foi negada ou cancelada")]
    ElevationDenied,
    #[error("falha ao iniciar o Serviço: {0}")]
    Other(String),
    #[error("plataforma não suportada")]
    UnsupportedPlatform,
}

pub(crate) fn resolve_app_data_dir() -> PathBuf {
    if let Ok(program_data) = std::env::var("ProgramData") {
        PathBuf::from(program_data).join("SystemForgeOptimizer")
    } else {
        std::env::temp_dir().join("systemforge-optimizer-dev")
    }
}

fn service_exe_path() -> Result<PathBuf, LaunchError> {
    let exe = std::env::current_exe().map_err(|e| LaunchError::Other(e.to_string()))?;
    let dir = exe.parent().ok_or(LaunchError::ExecutableNotFound)?;
    let candidate = dir.join("systemforge-service.exe");
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(LaunchError::ExecutableNotFound)
    }
}

#[cfg(windows)]
mod win_impl {
    use super::LaunchError;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_CANCELLED;
    use windows::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn launch_elevated(path: &std::path::Path) -> Result<(), LaunchError> {
        let path_wide = to_wide(&path.to_string_lossy());
        let verb_wide = to_wide("runas");

        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
            lpVerb: PCWSTR(verb_wide.as_ptr()),
            lpFile: PCWSTR(path_wide.as_ptr()),
            nShow: SW_HIDE.0 as i32,
            ..Default::default()
        };

        // ShellExecuteExW retorna sucesso/falha via BOOL; o motivo real (ex.:
        // usuário negou o UAC) vem de GetLastError quando falha.
        let result = unsafe { ShellExecuteExW(&mut info) };
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                    Err(LaunchError::ElevationDenied)
                } else {
                    Err(LaunchError::Other(e.message().to_string()))
                }
            }
        }
    }
}

#[cfg(not(windows))]
mod win_impl {
    use super::LaunchError;

    pub fn launch_elevated(_path: &std::path::Path) -> Result<(), LaunchError> {
        Err(LaunchError::UnsupportedPlatform)
    }
}

/// Dispara o Serviço elevado (via UAC) se ainda não tentamos nesta sessão do
/// Core. Não bloqueia esperando o Serviço terminar de subir — quem chamar
/// deve fazer polling separado com `wait_for_token`.
pub fn ensure_launched() -> Result<(), LaunchError> {
    if LAUNCH_ATTEMPTED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let path = service_exe_path()?;
    let outcome = win_impl::launch_elevated(&path);
    if outcome.is_err() {
        // Permite tentar de novo na próxima chamada (ex.: usuário corrigiu
        // algo e vai clicar em "Aplicar" outra vez).
        LAUNCH_ATTEMPTED.store(false, Ordering::SeqCst);
    }
    outcome
}

/// Espera até `timeout` pelo arquivo de token do Serviço aparecer,
/// checando a cada 200ms.
pub fn wait_for_token(timeout: Duration) -> bool {
    let token_path = resolve_app_data_dir().join(systemforge_shared::TOKEN_FILE_NAME);
    let start = Instant::now();
    loop {
        if token_path.exists() {
            return true;
        }
        if start.elapsed() >= timeout {
            return token_path.exists();
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
