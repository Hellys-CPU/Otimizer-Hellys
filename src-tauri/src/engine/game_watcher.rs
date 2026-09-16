//! Detector de jogos em background: a cada poll, verifica processos em
//! execução contra os jogos cadastrados com perfil associado.
//!
//! Início do jogo   -> captura métricas "before" -> aplica o perfil.
//! Fechamento do jogo -> captura métricas "after" -> restaura o perfil.
//!
//! Roda em uma thread própria (não bloqueia a UI) e reabre o estado do banco
//! via `AppHandle::state`, que é seguro de chamar de qualquer thread.

use super::{profile as profile_engine, telemetry};
use crate::db::Db;
use crate::modules::{games, monitor};
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::Manager;
use uuid::Uuid;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

pub fn spawn(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        // game_id -> game_session_id da sessão em andamento
        let mut active: HashMap<String, String> = HashMap::new();

        loop {
            std::thread::sleep(POLL_INTERVAL);

            let db = app_handle.state::<Db>();
            let conn = match db.0.lock() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let registered = match games::list_games(&conn) {
                Ok(g) => g,
                Err(_) => continue,
            };
            let running = monitor::running_process_names();
            let matches = games::match_running_games(&registered, &running);
            let matched_ids: HashSet<String> = matches.iter().map(|(g, _)| g.id.clone()).collect();

            for (game, _pid) in &matches {
                if active.contains_key(&game.id) {
                    continue;
                }
                let Some(profile_id) = game.profile_id.clone() else { continue };

                let game_session_id = Uuid::new_v4().to_string();
                if conn
                    .execute(
                        "INSERT INTO game_sessions (id, game_id) VALUES (?1, ?2)",
                        params![game_session_id, game.id],
                    )
                    .is_err()
                {
                    continue;
                }

                let _ = telemetry::capture(&conn, &game_session_id, "before", Some(&game.executable_name));

                if let Ok(exec_session) = profile_engine::apply_profile(&conn, &profile_id) {
                    let _ = conn.execute(
                        "UPDATE game_sessions SET execution_session_id = ?1 WHERE id = ?2",
                        params![exec_session.id, game_session_id],
                    );
                }
                let _ = games::mark_played_now(&conn, &game.id);

                active.insert(game.id.clone(), game_session_id);
            }

            let ended_game_ids: Vec<String> = active
                .keys()
                .filter(|id| !matched_ids.contains(*id))
                .cloned()
                .collect();

            for game_id in ended_game_ids {
                let Some(game_session_id) = active.remove(&game_id) else { continue };
                // O processo já saiu, então o PresentMon não vai achar nada — mas
                // ainda tentamos, para o caso raro de o jogo já ter reaberto sob
                // o mesmo executável antes deste poll rodar.
                let exe_name = registered.iter().find(|g| g.id == game_id).map(|g| g.executable_name.clone());
                let _ = telemetry::capture(&conn, &game_session_id, "after", exe_name.as_deref());
                let _ = conn.execute(
                    "UPDATE game_sessions SET status = 'finished', ended_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
                    [&game_session_id],
                );

                if let Some(game) = registered.iter().find(|g| g.id == game_id) {
                    if let Some(profile_id) = &game.profile_id {
                        let _ = profile_engine::restore_profile(&conn, profile_id);
                    }
                }
            }
        }
    });
}
