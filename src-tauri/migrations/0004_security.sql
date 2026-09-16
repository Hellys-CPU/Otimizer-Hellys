-- Catálogo de Segurança: tweaks que afetam recursos de proteção do Windows.
--
-- Propositalmente NÃO são vinculados a nenhum profile_optimizations — não
-- existe caminho para esses tweaks serem aplicados via "Aplicar perfil".
-- Só podem ser aplicados um a um, pela tela de Segurança, com confirmação
-- individual (é a mesma engine de registry_set_dword/registry_restore_value,
-- mas exposta apenas por essa tela).

INSERT INTO optimizations (
    id, nome, categoria, descricao, objetivo, beneficio_esperado, risco,
    requer_administrador, requer_reinicializacao,
    caminho, nome_do_valor, tipo_do_valor, valor_recomendado,
    comando_de_aplicacao, comando_de_reversao, fonte_tecnica
) VALUES
(
    'sec-disable-hvci',
    'Desativar Integridade de Memória (HVCI)',
    'security',
    'Desativa o Hypervisor-Enforced Code Integrity, que isola a verificação de código do kernel em um ambiente virtualizado.',
    'Reduzir a sobrecarga de virtualização em cargas de trabalho muito sensíveis a latência.',
    'Pode reduzir overhead de virtualização em alguns jogos/benchmarks; efeito varia por hardware e não é garantido.',
    'experimental', 1, 1,
    'HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity',
    'Enabled',
    'REG_DWORD',
    '0',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Memory integrity (HVCI)'
),
(
    'sec-disable-vbs',
    'Desativar Segurança Baseada em Virtualização (VBS)',
    'security',
    'Desativa a VBS, que usa virtualização de hardware para isolar processos sensíveis do kernel.',
    'Reduzir overhead de virtualização.',
    'Benefício de desempenho não garantido; remove uma camada de isolamento usada por Credential Guard e HVCI.',
    'experimental', 1, 1,
    'HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard',
    'EnableVirtualizationBasedSecurity',
    'REG_DWORD',
    '0',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Virtualization-based security (VBS)'
),
(
    'sec-disable-smartscreen-policy',
    'Desativar SmartScreen (política de sistema)',
    'security',
    'Desativa a verificação SmartScreen de aplicativos e arquivos baixados via política de grupo.',
    'Evitar checagens do SmartScreen ao abrir executáveis não assinados/baixados.',
    'Remove um aviso de segurança contra malware conhecido; nenhum ganho de desempenho.',
    'experimental', 1, 0,
    'HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System',
    'EnableSmartScreen',
    'REG_DWORD',
    '0',
    'registry_set_dword',
    'registry_restore_value',
    'Microsoft Learn — Microsoft Defender SmartScreen policy settings'
);

INSERT INTO compatibility_rules (id, optimization_id, windows_version, build_minimo) VALUES
('cr-11', 'sec-disable-hvci', '10', 17763),
('cr-12', 'sec-disable-hvci', '11', 22000),
('cr-13', 'sec-disable-vbs', '10', 17763),
('cr-14', 'sec-disable-vbs', '11', 22000),
('cr-15', 'sec-disable-smartscreen-policy', '10', 10240),
('cr-16', 'sec-disable-smartscreen-policy', '11', 22000);
