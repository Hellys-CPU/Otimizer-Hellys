-- Sessões de jogo + telemetria de desempenho antes/depois.
--
-- IMPORTANTE: tudo aqui é armazenado 100% localmente no SQLite do usuário.
-- Não existe, nesta fase, nenhum envio para servidor externo. `user_settings`
-- guarda `telemetry_opt_in` como flag para uma futura camada de envio remoto
-- opcional — hoje ela não é lida por nenhum código de rede porque esse código
-- não existe.

CREATE TABLE game_sessions (
    id                      TEXT PRIMARY KEY,
    game_id                 TEXT NOT NULL REFERENCES detected_games(id) ON DELETE CASCADE,
    execution_session_id    TEXT REFERENCES execution_sessions(id) ON DELETE SET NULL,
    started_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at                TEXT,
    status                  TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'finished'))
);

CREATE TABLE telemetry_events (
    id                  TEXT PRIMARY KEY,
    game_session_id     TEXT NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    phase               TEXT NOT NULL CHECK (phase IN ('before', 'after')),
    captured_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    fps_avg             REAL,
    fps_low1pct         REAL,
    cpu_usage_percent   REAL,
    gpu_usage_percent   REAL,
    temperature_c       REAL,
    ram_usage_mb        REAL,
    input_latency_ms    REAL,
    game_startup_ms     REAL
);

CREATE INDEX idx_game_sessions_game ON game_sessions(game_id);
CREATE INDEX idx_telemetry_events_session ON telemetry_events(game_session_id);

INSERT INTO user_settings (key, value) VALUES
    ('telemetry_opt_in', 'false'),
    ('telemetry_notice', 'Métricas de antes/depois ficam salvas apenas neste computador. Nenhum dado é enviado para a internet nesta versão.');
