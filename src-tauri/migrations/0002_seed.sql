-- Seed inicial: perfil padrão do Windows + 5 otimizações nível 1 (seguras, reversíveis)

INSERT INTO profiles (id, nome, descricao, icone, cor, nivel_agressividade, requer_administrador, is_padrao_windows)
VALUES ('profile-default-windows', 'Padrão do Windows', 'Nenhuma alteração aplicada; estado de fábrica do sistema.', 'shield', '#64748b', 1, 0, 1);

INSERT INTO profiles (id, nome, descricao, icone, cor, nivel_agressividade, requer_administrador)
VALUES ('profile-gaming-competitivo', 'Gaming competitivo', 'Prioriza responsividade e latência mínima para jogos competitivos.', 'gamepad', '#ef4444', 2, 1);

INSERT INTO optimizations (
    id, nome, categoria, descricao, objetivo, beneficio_esperado, risco,
    requer_administrador, requer_reinicializacao,
    caminho, nome_do_valor, tipo_do_valor, valor_recomendado,
    comando_de_aplicacao, comando_de_reversao, fonte_tecnica
) VALUES
(
    'opt-disable-tips',
    'Desativar dicas e sugestões do Windows',
    'registry',
    'Desativa notificações de sugestão de conteúdo/anúncios do sistema.',
    'Reduzir interrupções e notificações não essenciais.',
    'Menos distrações durante uso e jogos, sem impacto de desempenho mensurável.',
    'safe', 0, 0,
    'HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager',
    'SubscribedContent-338389Enabled',
    'REG_DWORD',
    '0',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Content Delivery Manager policies'
),
(
    'opt-startup-app-disable',
    'Desativar aplicativo de inicialização selecionado',
    'startup',
    'Impede que um aplicativo específico inicie junto com o Windows.',
    'Reduzir tempo de boot e consumo de RAM ocioso.',
    'Inicialização mais rápida; efeito varia por aplicativo.',
    'safe', 0, 0,
    NULL, NULL, NULL, NULL,
    'startup_disable_app',
    'startup_restore_app',
    'Windows StartupApproved\Run documentation'
),
(
    'opt-power-high-performance',
    'Plano de energia Alto Desempenho durante jogo',
    'power_plan',
    'Ativa o plano de energia de alto desempenho enquanto um jogo está em execução.',
    'Evitar throttling de CPU por gerenciamento agressivo de energia.',
    'Clocks de CPU mais estáveis sob carga; consumo de energia maior.',
    'safe', 0, 0,
    NULL, NULL, NULL, NULL,
    'power_set_scheme',
    'power_restore_scheme',
    'powercfg /list, /setactive (Microsoft Docs)'
),
(
    'opt-game-mode-on',
    'Habilitar Modo de Jogo do Windows',
    'registry',
    'Garante que o Modo de Jogo do Windows esteja ativo.',
    'Priorizar recursos do sistema para o jogo em primeiro plano.',
    'Redução de interrupções em segundo plano durante o jogo.',
    'safe', 0, 0,
    'HKEY_CURRENT_USER\Software\Microsoft\GameBar',
    'AllowAutoGameMode',
    'REG_DWORD',
    '1',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Game Mode'
),
(
    'opt-disable-background-apps',
    'Desativar apps em segundo plano (chave geral)',
    'registry',
    'Desativa a opção mestre "Permitir que apps sejam executados em segundo plano".',
    'Reduzir consumo de CPU/RAM ocioso por aplicativos UWP parados em segundo plano.',
    'Menos consumo de recursos em segundo plano; apps perdem atualização em tempo real (notificações, sincronização).',
    'safe', 0, 0,
    'HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications',
    'GlobalUserDisabled',
    'REG_DWORD',
    '1',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Background apps policy (GlobalUserDisabled)'
);

INSERT INTO profile_optimizations (profile_id, optimization_id, proibido) VALUES
('profile-gaming-competitivo', 'opt-disable-tips', 0),
('profile-gaming-competitivo', 'opt-startup-app-disable', 0),
('profile-gaming-competitivo', 'opt-power-high-performance', 0),
('profile-gaming-competitivo', 'opt-game-mode-on', 0),
('profile-gaming-competitivo', 'opt-disable-background-apps', 0);

INSERT INTO compatibility_rules (id, optimization_id, windows_version, build_minimo) VALUES
('cr-1', 'opt-disable-tips', '10', 10240),
('cr-2', 'opt-disable-tips', '11', 22000),
('cr-3', 'opt-startup-app-disable', '10', 10240),
('cr-4', 'opt-startup-app-disable', '11', 22000),
('cr-5', 'opt-power-high-performance', '10', 10240),
('cr-6', 'opt-power-high-performance', '11', 22000),
('cr-7', 'opt-game-mode-on', '10', 17134),
('cr-8', 'opt-game-mode-on', '11', 22000),
('cr-9', 'opt-disable-background-apps', '10', 10240),
('cr-10', 'opt-disable-background-apps', '11', 22000);
