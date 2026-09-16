use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
    ("0002_seed", include_str!("../../migrations/0002_seed.sql")),
    ("0003_telemetry", include_str!("../../migrations/0003_telemetry.sql")),
    ("0004_security", include_str!("../../migrations/0004_security.sql")),
    ("0005_advanced", include_str!("../../migrations/0005_advanced.sql")),
    ("0006_profiles", include_str!("../../migrations/0006_profiles.sql")),
];

impl Db {
    pub fn open(app_data_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("systemforge.sqlite");
        let mut conn = Connection::open(db_path)?;
        conn.pragma_update(None, "foreign_keys", true)?;
        run_migrations(&mut conn)?;
        Ok(Db(Mutex::new(conn)))
    }

    #[cfg(test)]
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", true)?;
        run_migrations(&mut conn)?;
        Ok(Db(Mutex::new(conn)))
    }
}

fn run_migrations(conn: &mut Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')))",
    )?;

    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?1)",
                [name],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if already_applied {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute("INSERT INTO schema_migrations (name) VALUES (?1)", [name])?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_cleanly_and_seed_is_present() {
        let db = Db::open_in_memory().expect("migrations devem aplicar sem erro");
        let conn = db.0.lock().unwrap();

        let optimizations: i64 = conn
            .query_row("SELECT COUNT(*) FROM optimizations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(optimizations, 21, "5 seed + 3 segurança + 13 avançado (0005)");

        let security_optimizations: i64 = conn
            .query_row("SELECT COUNT(*) FROM optimizations WHERE categoria = 'security'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(security_optimizations, 4);

        let experimental_linked_to_profiles: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM profile_optimizations po
                 JOIN optimizations o ON o.id = po.optimization_id
                 WHERE o.risco = 'experimental'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            experimental_linked_to_profiles, 0,
            "nível 3 (experimental) nunca pode ser aplicado via perfil, só individualmente"
        );

        let profiles: i64 = conn
            .query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))
            .unwrap();
        assert_eq!(profiles, 6);

        let telemetry_opt_in: String = conn
            .query_row("SELECT value FROM user_settings WHERE key = 'telemetry_opt_in'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(telemetry_opt_in, "false", "telemetria deve vir desativada por padrão");

        // tabelas de telemetria/sessão de jogo devem existir e estar vazias
        let sessions: i64 = conn.query_row("SELECT COUNT(*) FROM game_sessions", [], |r| r.get(0)).unwrap();
        assert_eq!(sessions, 0);
    }

    #[test]
    fn migrations_are_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        run_migrations(&mut conn).unwrap(); // segunda chamada não deve reaplicar nem falhar
        let optimizations: i64 = conn
            .query_row("SELECT COUNT(*) FROM optimizations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(optimizations, 21);
    }
}
