//! Cadastro de jogos detectáveis (Steam, Epic, Xbox, Battle.net, Riot ou
//! executável personalizado) e associação a um perfil. A detecção em si
//! (varredura de processos) é feita pelo engine/game_watcher.rs usando
//! `modules::monitor::running_process_names`; este módulo só cuida do CRUD.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGame {
    pub id: String,
    pub nome: String,
    pub executable_name: String,
    pub executable_path: Option<String>,
    pub launcher: Option<String>,
    pub profile_id: Option<String>,
    pub last_played_at: Option<String>,
}

pub const KNOWN_LAUNCHERS: &[&str] = &["steam", "epic", "xbox", "battlenet", "riot", "custom"];

pub fn list_games(conn: &Connection) -> rusqlite::Result<Vec<DetectedGame>> {
    let mut stmt = conn.prepare(
        "SELECT id, nome, executable_name, executable_path, launcher, profile_id, last_played_at
         FROM detected_games ORDER BY nome",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DetectedGame {
                id: row.get(0)?,
                nome: row.get(1)?,
                executable_name: row.get(2)?,
                executable_path: row.get(3)?,
                launcher: row.get(4)?,
                profile_id: row.get(5)?,
                last_played_at: row.get(6)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(rows)
}

pub fn register_game(
    conn: &Connection,
    nome: &str,
    executable_name: &str,
    launcher: Option<&str>,
    profile_id: Option<&str>,
) -> rusqlite::Result<String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO detected_games (id, nome, executable_name, launcher, profile_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, nome, executable_name.to_lowercase(), launcher, profile_id],
    )?;
    Ok(id)
}

pub fn set_game_profile(conn: &Connection, game_id: &str, profile_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE detected_games SET profile_id = ?1 WHERE id = ?2",
        params![profile_id, game_id],
    )?;
    Ok(())
}

pub fn delete_game(conn: &Connection, game_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM detected_games WHERE id = ?1", [game_id])?;
    Ok(())
}

pub fn mark_played_now(conn: &Connection, game_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE detected_games SET last_played_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [game_id],
    )?;
    Ok(())
}

/// Casa a lista de processos em execução contra os jogos cadastrados que têm
/// um perfil associado. Comparação por nome de executável, case-insensitive.
pub fn match_running_games(
    games: &[DetectedGame],
    running_processes: &[(u32, String)],
) -> Vec<(DetectedGame, u32)> {
    let mut matches = Vec::new();
    for game in games {
        if game.profile_id.is_none() {
            continue;
        }
        if let Some((pid, _)) = running_processes
            .iter()
            .find(|(_, name)| name.to_lowercase() == game.executable_name)
        {
            matches.push((game.clone(), *pid));
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: &str, exe: &str, profile: Option<&str>) -> DetectedGame {
        DetectedGame {
            id: id.into(),
            nome: id.into(),
            executable_name: exe.into(),
            executable_path: None,
            launcher: Some("steam".into()),
            profile_id: profile.map(String::from),
            last_played_at: None,
        }
    }

    #[test]
    fn matches_case_insensitively() {
        let games = vec![game("g1", "csgo.exe", Some("profile-a"))];
        let running = vec![(1234, "CSGO.exe".to_string())];
        let matches = match_running_games(&games, &running);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1, 1234);
    }

    #[test]
    fn ignores_games_without_profile() {
        let games = vec![game("g1", "csgo.exe", None)];
        let running = vec![(1234, "csgo.exe".to_string())];
        assert!(match_running_games(&games, &running).is_empty());
    }

    #[test]
    fn ignores_processes_that_dont_match() {
        let games = vec![game("g1", "csgo.exe", Some("profile-a"))];
        let running = vec![(1234, "explorer.exe".to_string())];
        assert!(match_running_games(&games, &running).is_empty());
    }
}
