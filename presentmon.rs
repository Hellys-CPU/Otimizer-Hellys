//! Métricas de FPS reais via PresentMon (ferramenta open-source da Intel:
//! https://github.com/GameTechDev/PresentMon), em vez de reimplementar um
//! consumidor ETW/DXGI do zero — muito menor risco de entregar algo quebrado
//! sem uma máquina Windows real para validar contra um jogo rodando.
//!
//! Se o executável do PresentMon não estiver presente, `capture` retorna
//! `Ok(None)` — a UI mostra "não disponível", nunca um valor inventado.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresentMonError {
    #[error("nome de processo inválido: {0}")]
    InvalidProcessName(String),
    #[error("falha ao executar o PresentMon: {0}")]
    CommandFailed(String),
    #[error("não foi possível ler o CSV de saída do PresentMon: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FpsSample {
    pub fps_avg: f64,
    pub fps_low1pct: f64,
    pub frame_count: usize,
}

fn validate_process_name(name: &str) -> Result<(), PresentMonError> {
    let valid = !name.is_empty()
        && name.len() <= 260
        && name.chars().all(|c| !c.is_control() && c != '"' && c != '&' && c != '|');
    if valid {
        Ok(())
    } else {
        Err(PresentMonError::InvalidProcessName(name.to_string()))
    }
}

/// Procura o executável do PresentMon em `PATH` — o usuário precisa instalá-lo
/// separadamente (não é redistribuído com o SystemForge Optimizer).
pub fn locate() -> Option<PathBuf> {
    let candidates = ["PresentMon64-1.10.0.exe", "PresentMon64.exe", "PresentMon.exe"];
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in candidates {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Captura FPS do processo `process_name` (ex.: "csgo.exe") por `duration`,
/// escrevendo um CSV temporário e o interpretando. Retorna `Ok(None)` se o
/// PresentMon não estiver instalado — nunca inventa um número.
pub fn capture(process_name: &str, duration: Duration) -> Result<Option<FpsSample>, PresentMonError> {
    validate_process_name(process_name)?;

    let Some(exe) = locate() else { return Ok(None) };

    let output_path = std::env::temp_dir().join(format!(
        "systemforge-presentmon-{}.csv",
        uuid::Uuid::new_v4()
    ));

    let status = Command::new(&exe)
        .args([
            "-process_name",
            process_name,
            "-output_file",
            output_path.to_string_lossy().as_ref(),
            "-timed",
            &duration.as_secs().to_string(),
            "-stop_existing_session",
            "-terminate_after_timed",
            "-no_top",
        ])
        .status()
        .map_err(|e| PresentMonError::CommandFailed(e.to_string()))?;

    if !status.success() {
        let _ = std::fs::remove_file(&output_path);
        return Ok(None);
    }

    let sample = parse_csv_file(&output_path)?;
    let _ = std::fs::remove_file(&output_path);
    Ok(sample)
}

fn parse_csv_file(path: &Path) -> Result<Option<FpsSample>, PresentMonError> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_csv(&content))
}

/// Interpreta o CSV do PresentMon (formato com coluna `MsBetweenPresents`).
/// Função pura, deliberadamente separada de I/O para ser testável sem o
/// binário real do PresentMon.
fn parse_csv(content: &str) -> Option<FpsSample> {
    let mut lines = content.lines();
    let header = lines.next()?;
    let col_idx = header
        .split(',')
        .position(|c| c.trim().eq_ignore_ascii_case("MsBetweenPresents"))?;

    let mut frame_times_ms: Vec<f64> = lines
        .filter_map(|line| line.split(',').nth(col_idx))
        .filter_map(|v| v.trim().parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .collect();

    if frame_times_ms.is_empty() {
        return None;
    }

    let fps_values: Vec<f64> = frame_times_ms.iter().map(|ms| 1000.0 / ms).collect();
    let fps_avg = fps_values.iter().sum::<f64>() / fps_values.len() as f64;

    frame_times_ms.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let worst_1pct_count = ((frame_times_ms.len() as f64) * 0.01).ceil().max(1.0) as usize;
    let worst_slice = &frame_times_ms[..worst_1pct_count.min(frame_times_ms.len())];
    let avg_worst_ms = worst_slice.iter().sum::<f64>() / worst_slice.len() as f64;
    let fps_low1pct = 1000.0 / avg_worst_ms;

    Some(FpsSample {
        fps_avg,
        fps_low1pct,
        frame_count: frame_times_ms.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_process_name_with_shell_metacharacters() {
        assert!(validate_process_name("game.exe\" & calc.exe").is_err());
    }

    #[test]
    fn accepts_ordinary_process_name() {
        assert!(validate_process_name("csgo.exe").is_ok());
    }

    #[test]
    fn parses_well_formed_csv() {
        let csv = "Application,ProcessID,MsBetweenPresents\n\
                    game.exe,1234,16.6\n\
                    game.exe,1234,16.7\n\
                    game.exe,1234,100.0\n"; // um frame ruim, empurra o 1% low
        let sample = parse_csv(csv).unwrap();
        assert!(sample.fps_avg > 0.0);
        assert_eq!(sample.frame_count, 3);
        // o 1% low deve ser puxado pelo frame de 100ms (10 fps)
        assert!(sample.fps_low1pct < sample.fps_avg);
    }

    #[test]
    fn returns_none_for_csv_without_expected_column() {
        let csv = "Foo,Bar\n1,2\n";
        assert!(parse_csv(csv).is_none());
    }

    #[test]
    fn returns_none_for_empty_data_rows() {
        let csv = "Application,ProcessID,MsBetweenPresents\n";
        assert!(parse_csv(csv).is_none());
    }

    #[test]
    fn ignores_non_numeric_and_zero_rows() {
        let csv = "MsBetweenPresents\nnot-a-number\n0\n16.6\n";
        let sample = parse_csv(csv).unwrap();
        assert_eq!(sample.frame_count, 1);
    }
}
