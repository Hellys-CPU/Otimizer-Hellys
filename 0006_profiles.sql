-- Perfis adicionais da lista original: Gaming baixa latência, Streaming,
-- Trabalho e desenvolvimento (foco em apps pesados como planilhas/IDEs),
-- Economia de energia.

INSERT INTO profiles (id, nome, descricao, icone, cor, nivel_agressividade, requer_administrador)
VALUES
(
    'profile-gaming-baixa-latencia',
    'Gaming baixa latência',
    'Prioriza resposta de CPU/GPU e reduz limitação de rede — mais agressivo que o Gaming competitivo em ajustes de sistema.',
    'zap', '#ef4444', 2, 1
),
(
    'profile-streaming',
    'Streaming',
    'Plano de energia equilibrado e menos ruído de segundo plano, sem mexer em prioridade de CPU/GPU (evita cortar o encoder).',
    'video', '#a855f7', 1, 0
),
(
    'profile-trabalho',
    'Trabalho e desenvolvimento',
    'Prioriza o app em primeiro plano (IDE, planilha pesada) e reduz interrupções, mantendo consumo equilibrado.',
    'briefcase', '#3b82f6', 2, 1
),
(
    'profile-economia',
    'Economia de energia',
    'Reduz consumo e notificações — recomendado para notebook na bateria.',
    'battery', '#22c55e', 1, 0
);

INSERT INTO profile_optimizations (profile_id, optimization_id, proibido) VALUES
-- Gaming baixa latência
('profile-gaming-baixa-latencia', 'opt-disable-tips', 0),
('profile-gaming-baixa-latencia', 'opt-power-high-performance', 0),
('profile-gaming-baixa-latencia', 'opt-game-mode-on', 0),
('profile-gaming-baixa-latencia', 'opt-network-throttling-off', 0),
('profile-gaming-baixa-latencia', 'opt-system-responsiveness', 0),
('profile-gaming-baixa-latencia', 'opt-win32-priority-separation', 0),
('profile-gaming-baixa-latencia', 'opt-games-gpu-priority', 0),
('profile-gaming-baixa-latencia', 'opt-disable-gamebar-capture', 0),

-- Streaming
('profile-streaming', 'opt-disable-tips', 0),
('profile-streaming', 'opt-power-balanced', 0),
('profile-streaming', 'opt-disable-background-apps', 0),

-- Trabalho e desenvolvimento
('profile-trabalho', 'opt-disable-tips', 0),
('profile-trabalho', 'opt-win32-priority-separation', 0),
('profile-trabalho', 'opt-system-responsiveness', 0),
('profile-trabalho', 'opt-power-balanced', 0),
('profile-trabalho', 'opt-disable-background-apps', 0),

-- Economia de energia
('profile-economia', 'opt-power-saver', 0),
('profile-economia', 'opt-disable-tips', 0),
('profile-economia', 'opt-disable-background-apps', 0);
