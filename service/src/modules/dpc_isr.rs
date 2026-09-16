//! Captura de trace de kernel para análise de DPC/ISR via `wpr.exe`
//! (Windows Performance Recorder — built-in em qualquer Windows 10/11,
//! em `System32\wpr.exe`, não precisa instalar nada para gravar).
//!
//! A geração do resumo por driver (`xperf -a dpcisr`) só roda se o usuário
//! tiver o Windows ADK instalado (`xperf.exe` não vem com o Windows). Esse
//! parser de CSV NUNCA foi validado contra uma saída real do xperf — não
//! temos Windows disponível neste ambiente de desenvolvimento para gerar um
//! trace de verdade. Por isso, em vez de arriscar um parser errado e exibir
//! números fabricados, devolvemos os caminhos do `.etl`/`.csv` brutos: quem
//! tiver o Windows Performance Analyzer (grátis, Microsoft Store) abre e
//! analisa visualmente, que é como profissionais fazem essa análise mesmo.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DpcIsrError {
    #[error("falha ao executar wpr: {0}")]
    CommandFailed(String),
    #[error("wpr -start falhou: {0}")]
    StartFailed(String),
    #[error("wpr -stop falhou: {0}")]
    StopFailed(String),
}

pub struct CaptureResult {
    pub etl_path: PathBuf,
    pub csv_path: Option<PathBuf>,
}

pub fn capture(duration: Duration) -> Result<CaptureResult, DpcIsrError> {
    let etl_path = std::env::temp_dir().join(format!("systemforge-dpcisr-{}.etl", uuid_like()));

    let start = Command::new("wpr")
        .args(["-start", "CPU", "-filemode"])
        .output()
        .map_err(|e| DpcIsrError::CommandFailed(e.to_string()))?;
    if !start.status.success() {
        return Err(DpcIsrError::StartFailed(String::from_utf8_lossy(&start.stderr).trim().to_string()));
    }

    std::thread::sleep(duration);

    let stop = Command::new("wpr")
        .args(["-stop", &etl_path.to_string_lossy()])
        .output()
        .map_err(|e| DpcIsrError::CommandFailed(e.to_string()))?;
    if !stop.status.success() {
        return Err(DpcIsrError::StopFailed(String::from_utf8_lossy(&stop.stderr).trim().to_string()));
    }

    let csv_path = try_summarize_with_xperf(&etl_path);

    Ok(CaptureResult { etl_path, csv_path })
}

/// Só roda se `xperf.exe` estiver no PATH (Windows ADK). Se falhar por
/// qualquer motivo, retorna `None` silenciosamente — o `.etl` bruto já
/// gravado continua válido de qualquer forma.
fn try_summarize_with_xperf(etl_path: &std::path::Path) -> Option<PathBuf> {
    let csv_path = etl_path.with_extension("dpcisr.csv");
    let result = Command::new("xperf")
        .args([
            "-i",
            &etl_path.to_string_lossy(),
            "-o",
            &csv_path.to_string_lossy(),
            "-a",
            "dpcisr",
        ])
        .output()
        .ok()?;
    if result.status.success() && csv_path.exists() {
        Some(csv_path)
    } else {
        None
    }
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{nanos:x}")
}
