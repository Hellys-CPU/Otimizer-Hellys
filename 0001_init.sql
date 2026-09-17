-- SystemForge Optimizer — schema inicial
-- Convenção: IDs de texto são UUID v4; timestamps são TEXT ISO-8601 UTC.

PRAGMA foreign_keys = ON;

CREATE TABLE profiles (
    id                      TEXT PRIMARY KEY,
    nome                    TEXT NOT NULL,
    descricao               TEXT NOT NULL DEFAULT '',
    icone                   TEXT NOT NULL DEFAULT '',
    cor                     TEXT NOT NULL DEFAULT '#3b82f6',
    nivel_agressividade     INTEGER NOT NULL CHECK (nivel_agressividade IN (1, 2, 3)),
    requer_administrador    INTEGER NOT NULL DEFAULT 0,
    ativo                   INTEGER NOT NULL DEFAULT 0,
    ultima_aplicacao        TEXT,
    is_padrao_windows       INTEGER NOT NULL DEFAULT 0,
    is_personalizado        INTEGER NOT NULL DEFAULT 0,
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE optimizations (
    id                      TEXT PRIMARY KEY,
    nome                    TEXT NOT NULL,
    categoria               TEXT NOT NULL CHECK (categoria IN (
                                'registry', 'service', 'scheduled_task', 'startup',
                                'power_plan', 'network', 'security', 'gaming', 'ui'
                            )),
    descricao               TEXT NOT NULL DEFAULT '',
    objetivo                TEXT NOT NULL DEFAULT '',
    beneficio_esperado      TEXT NOT NULL DEFAULT '',
    risco                   TEXT NOT NULL CHECK (risco IN ('safe', 'advanced', 'experimental')),
    requer_administrador    INTEGER NOT NULL DEFAULT 0,
    requer_reinicializacao  INTEGER NOT NULL DEFAULT 0,

    -- específico de tweak de registro (NULL para outras categorias)
    caminho                 TEXT,
    nome_do_valor           TEXT,
    tipo_do_valor           TEXT CHECK (tipo_do_valor IN ('REG_DWORD','REG_SZ','REG_QWORD','REG_BINARY','REG_MULTI_SZ')),
    valor_recomendado       TEXT,
    valor_atual             TEXT,

    comando_de_aplicacao    TEXT NOT NULL, -- identificador do handler Rust registrado, nunca comando de shell
    comando_de_reversao     TEXT NOT NULL,
    fonte_tecnica           TEXT NOT NULL DEFAULT '',
    ativo                   INTEGER NOT NULL DEFAULT 1,
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE profile_optimizations (
    profile_id       TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    optimization_id  TEXT NOT NULL REFERENCES optimizations(id) ON DELETE RESTRICT,
    proibido         INTEGER NOT NULL DEFAULT 0, -- true = está na lista de otimizações proibidas do perfil
    PRIMARY KEY (profile_id, optimization_id)
);

CREATE TABLE compatibility_rules (
    id                TEXT PRIMARY KEY,
    optimization_id   TEXT NOT NULL REFERENCES optimizations(id) ON DELETE CASCADE,
    windows_version    TEXT NOT NULL CHECK (windows_version IN ('10', '11')),
    build_minimo       INTEGER,
    build_maximo       INTEGER,
    requer_recurso     TEXT -- ex.: 'tpm', 'vbs', 'secure_boot'
);

CREATE TABLE system_capabilities (
    id                TEXT PRIMARY KEY,
    detected_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    windows_version     TEXT NOT NULL,
    windows_build       INTEGER NOT NULL,
    windows_edition     TEXT,
    tpm_disponivel      INTEGER NOT NULL DEFAULT 0,
    vbs_disponivel      INTEGER NOT NULL DEFAULT 0,
    secure_boot_ativo   INTEGER NOT NULL DEFAULT 0,
    is_notebook         INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE execution_sessions (
    id            TEXT PRIMARY KEY,
    profile_id    TEXT REFERENCES profiles(id) ON DELETE SET NULL,
    started_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    finished_at   TEXT,
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN
                    ('pending','applied','partial','failed','restored')),
    restore_point_created INTEGER NOT NULL DEFAULT 0,
    restore_point_error   TEXT
);

CREATE TABLE optimization_snapshots (
    id                TEXT PRIMARY KEY,
    session_id        TEXT NOT NULL REFERENCES execution_sessions(id) ON DELETE RESTRICT,
    optimization_id   TEXT NOT NULL REFERENCES optimizations(id) ON DELETE RESTRICT,
    valor_original     TEXT, -- JSON serializado, genérico por tipo de tweak
    valor_aplicado      TEXT,
    aplicado_em         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    restaurado_em       TEXT,
    status              TEXT NOT NULL DEFAULT 'applied' CHECK (status IN ('applied','restored','failed'))
);

CREATE TABLE registry_backups (
    id                TEXT PRIMARY KEY,
    snapshot_id        TEXT NOT NULL REFERENCES optimization_snapshots(id) ON DELETE CASCADE,
    caminho            TEXT NOT NULL,
    nome_do_valor      TEXT NOT NULL,
    tipo_do_valor      TEXT NOT NULL,
    valor_original_raw TEXT, -- NULL = a chave/valor não existia antes (deve ser removida na restauração)
    existia            INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE service_snapshots (
    id                  TEXT PRIMARY KEY,
    snapshot_id          TEXT NOT NULL REFERENCES optimization_snapshots(id) ON DELETE CASCADE,
    service_name          TEXT NOT NULL,
    start_type_original    TEXT NOT NULL, -- Automatic | AutomaticDelayed | Manual | Disabled
    status_original         TEXT,
    dependencies_json        TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE scheduled_task_snapshots (
    id             TEXT PRIMARY KEY,
    snapshot_id     TEXT NOT NULL REFERENCES optimization_snapshots(id) ON DELETE CASCADE,
    task_path        TEXT NOT NULL,
    was_enabled       INTEGER NOT NULL,
    task_xml_backup   TEXT NOT NULL
);

CREATE TABLE power_plan_snapshots (
    id                TEXT PRIMARY KEY,
    snapshot_id        TEXT NOT NULL REFERENCES optimization_snapshots(id) ON DELETE CASCADE,
    active_scheme_guid  TEXT NOT NULL,
    scheme_settings_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE startup_snapshots (
    id                TEXT PRIMARY KEY,
    snapshot_id        TEXT NOT NULL REFERENCES optimization_snapshots(id) ON DELETE CASCADE,
    app_name            TEXT NOT NULL,
    executable_path       TEXT,
    was_enabled           INTEGER NOT NULL,
    location               TEXT NOT NULL -- registry run key | startup folder | task scheduler
);

CREATE TABLE execution_logs (
    id             TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES execution_sessions(id) ON DELETE CASCADE,
    command          TEXT NOT NULL,
    user_name         TEXT NOT NULL,
    timestamp         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    result            TEXT NOT NULL CHECK (result IN ('success', 'error')),
    error_message      TEXT,
    return_code        INTEGER,
    requires_reboot     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE detected_games (
    id                TEXT PRIMARY KEY,
    nome               TEXT NOT NULL,
    executable_name      TEXT NOT NULL,
    executable_path       TEXT,
    launcher            TEXT CHECK (launcher IN ('steam','epic','xbox','battlenet','riot','custom')),
    profile_id          TEXT REFERENCES profiles(id) ON DELETE SET NULL,
    last_played_at        TEXT
);

CREATE TABLE user_settings (
    key      TEXT PRIMARY KEY,
    value    TEXT NOT NULL
);

CREATE INDEX idx_snapshots_session ON optimization_snapshots(session_id);
CREATE INDEX idx_snapshots_optimization ON optimization_snapshots(optimization_id);
CREATE INDEX idx_logs_session ON execution_logs(session_id);
CREATE INDEX idx_profile_optimizations_profile ON profile_optimizations(profile_id);
CREATE INDEX idx_compatibility_rules_optimization ON compatibility_rules(optimization_id);
