//! Captura de métricas antes/depois de uma sessão de jogo.
//!
//! Só grava o que é realmente mensurável sem instrumentação adicional (CPU,
//! RAM). FPS, GPU, temperatura e latência de entrada exigem hooks
//! específicos (ETW/DXGI) não implementados nesta fase — ficam `NULL` no
//! banco e a UI deve exibi-los como "não disponível", nunca inventar um
//! valor.

use crate::modules::{monitor, presentmon};
use rusqlite::{params, Connection};
use std::time::Duration;
use uuid::Uuid;

/// Captura CPU/RAM (sempre) e, se `process_name` for informado e o PresentMon
/// estiver instalado, também FPS médio e 1% low reais (janela curta de 3s).
/// Quando o PresentMon não está disponível, os campos de FPS ficam `NULL` —
/// a UI mostra "não disponível", nunca um número inventado.
pub fn capture(
    conn: &Connection,
    game_session_id: &str,
    phase: &str,
    process_name: Option<&str>,
) -> rusqlite::Result<()> {
    let snap = monitor::snapshot(5);
    let fps = process_name
        .and_then(|name| presentmon::capture(name, Duration::from_secs(3)).ok().flatten());

    conn.execute(
        "INSERT INTO telemetry_events (id, game_session_id, phase, cpu_usage_percent, ram_usage_mb, fps_avg, fps_low1pct)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            game_session_id,
            phase,
            snap.cpu_usage as f64,
            snap.ram_used_mb as f64,
            fps.as_ref().map(|f| f.fps_avg),
            fps.as_ref().map(|f| f.fps_low1pct),
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryComparison {
    pub game_session_id: String,
    pub game_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub before_cpu_usage_percent: Option<f64>,
    pub after_cpu_usage_percent: Option<f64>,
    pub before_ram_usage_mb: Option<f64>,
    pub after_ram_usage_mb: Option<f64>,
    pub before_fps_avg: Option<f64>,
    pub after_fps_avg: Option<f64>,
    pub before_fps_low1pct: Option<f64>,
    pub after_fps_low1pct: Option<f64>,
}

pub fn list_comparisons(conn: &Connection) -> rusqlite::Result<Vec<TelemetryComparison>> {
    let mut stmt = conn.prepare(
        "SELECT gs.id, dg.nome, gs.started_at, gs.ended_at,
                (SELECT cpu_usage_percent FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'before' ORDER BY captured_at LIMIT 1),
                (SELECT cpu_usage_percent FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'after' ORDER BY captured_at DESC LIMIT 1),
                (SELECT ram_usage_mb FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'before' ORDER BY captured_at LIMIT 1),
                (SELECT ram_usage_mb FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'after' ORDER BY captured_at DESC LIMIT 1),
                (SELECT fps_avg FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'before' ORDER BY captured_at LIMIT 1),
                (SELECT fps_avg FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'after' ORDER BY captured_at DESC LIMIT 1),
                (SELECT fps_low1pct FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'before' ORDER BY captured_at LIMIT 1),
                (SELECT fps_low1pct FROM telemetry_events WHERE game_session_id = gs.id AND phase = 'after' ORDER BY captured_at DESC LIMIT 1)
         FROM game_sessions gs
         JOIN detected_games dg ON dg.id = gs.game_id
         ORDER BY gs.started_at DESC LIMIT 50",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TelemetryComparison {
                game_session_id: row.get(0)?,
                game_name: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                before_cpu_usage_percent: row.get(4)?,
                after_cpu_usage_percent: row.get(5)?,
                before_ram_usage_mb: row.get(6)?,
                after_ram_usage_mb: row.get(7)?,
                before_fps_avg: row.get(8)?,
                after_fps_avg: row.get(9)?,
                before_fps_low1pct: row.get(10)?,
                after_fps_low1pct: row.get(11)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(rows)
}
