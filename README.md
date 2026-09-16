# SystemForge Optimizer

Plataforma de otimização profunda, baseada em perfis e reversível, para Windows 10/11.
Ver a documentação de arquitetura completa na conversa de design deste projeto.

## Status

Arquitetura de dois processos implementada: **Core** (Tauri, sem privilégio elevado)
e **Serviço privilegiado** (`systemforge-service`, processo separado que é o único a
gravar no Registro/Serviços/Tarefas Agendadas/Rede/Energia), comunicando por TCP
loopback autenticado por token. Schema SQLite completo (15+ tabelas), motor de
perfis, motor de backup/restauração, monitor de hardware real, detecção de jogos
com aplicação/restauração automática, telemetria local (CPU/RAM sempre reais; FPS
real via PresentMon quando instalado) e catálogo de Segurança separado (nunca
aplicado via perfil, só individualmente).

**Este projeto só compila e roda de fato no Windows.** Os módulos que tocam Win32
(Registro, SCM de Serviços) usam a crate `windows` e só compilam com `cfg(windows)`;
fora do Windows eles retornam `UnsupportedPlatform`. Isso foi desenvolvido em um
ambiente Linux sem acesso a uma máquina Windows — o que pôde ser testado, foi:
- `shared`: framing do protocolo IPC, allowlist de ações, validação de entrada (path
  traversal, serviços protegidos, injeção de argumento) — 12 testes.
- `service`: toda a lógica que não é Win32 (schtasks/netsh via subprocesso com argv
  fixo), e **o protocolo IPC completo ponta a ponta** — o servidor real roda em
  TCP loopback e é testado com um cliente real conectando, autenticando e sendo
  rejeitado/aceito — 15 testes, incluindo um serviço protegido sendo recusado
  antes de qualquer chamada Win32.
- `src-tauri`: banco/migrações, motor de perfis, detecção de jogos, parsing do CSV
  do PresentMon — 11 testes (excluindo os arquivos que dependem do próprio Tauri,
  que não compila neste ambiente por falta de `webkit2gtk`/GTK do Linux).
- Frontend: `tsc --noEmit` e `vite build` de produção, ambos limpos.

O que **não pôde** ser testado (precisa de validação manual em uma máquina Windows
antes de produção): todas as chamadas Win32 propriamente ditas (Registro, Service
Control Manager), e o PresentMon (a lógica de parsing do CSV é testada com dados
sintéticos, mas nunca rodou contra o binário real).

## Rodando no Windows

```powershell
npm install
npm run tauri dev
```

Isso só sobe o Core. **O Serviço privilegiado precisa ser iniciado à parte** (não
há handshake de elevação/auto-start implementado ainda — ver "O que falta" abaixo):

```powershell
cargo run -p systemforge-service
```

Rode-o como Administrador (clique direito no terminal → "Executar como
administrador") para as operações de Registro/Serviços realmente terem permissão.
Sem o Serviço rodando, qualquer tentativa de aplicar um tweak retorna erro de
conexão — os dados de leitura (dashboard, catálogo, perfis) funcionam normalmente
sem ele.

Pré-requisitos: Rust (toolchain MSVC), Node 18+, [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
(já vem instalado por padrão no Windows 11 e na maioria das instalações do Windows 10 atualizadas).
Opcional: [PresentMon](https://github.com/GameTechDev/PresentMon) no PATH, para FPS real na tela de Comparação.

## Ícones do aplicativo

`src-tauri/icons/` já tem um ícone placeholder gerado (`npm run tauri icon`).
Troque por um logo de verdade quando tiver um, rodando o mesmo comando com o
arquivo fonte.

## Distribuição — gerar o instalador para baixar, sem precisar de Windows local

`.github/workflows/build-windows.yml` builda em um runner Windows real do
GitHub Actions (é assim que este projeto resolve não ter uma máquina Windows
disponível durante o desenvolvimento — os testes Rust rodam de verdade lá,
inclusive o protocolo IPC ponta a ponta e a compilação Win32 completa, nunca
testados neste ambiente Linux).

- Todo push/PR: só valida (testes + `tsc` + build) — não gera Release.
- Push de uma tag `v*` (ex.: `git tag v0.1.0 && git push origin v0.1.0`):
  builda o instalador e publica um **GitHub Release** com o `.exe`/`.msi`
  anexado, pronto para qualquer pessoa baixar.

O workflow também sobe os artefatos em toda execução (aba "Actions" → run →
"Artifacts"), então dá para pegar um build de teste mesmo sem criar uma tag.

## Estrutura

```
src/                     Frontend React + TypeScript
shared/                  Protocolo IPC + allowlist de ações + validação — usado
                          pelos dois processos abaixo, nenhum dos dois confia
                          cegamente no outro (cada lado valida de novo)
src-tauri/                Core (Tauri) — sem privilégio elevado
  src/db/                  Conexão SQLite + runner de migrations
  src/models.rs             Structs compartilhadas com o frontend (serde)
  src/modules/              registry.rs (só leitura), monitor.rs, games.rs,
                             presentmon.rs, service_client.rs (fala com o Serviço)
  src/engine/                profile.rs, backup.rs, game_watcher.rs, telemetry.rs
  src/commands/               Comandos Tauri expostos à UI (única superfície de IPC)
  migrations/                  SQL versionado (0001_init … 0004_security)
service/                  Serviço privilegiado — único processo que grava de fato
  src/transport.rs           TCP loopback + token de autenticação
  src/server.rs                Servidor: handshake de auth + loop de dispatch
  src/dispatch.rs                Mapeia ServiceRequest -> handlers
  src/modules/                    registry.rs, power.rs, startup.rs (escrita),
                                   services.rs (SCM), scheduled_tasks.rs (schtasks),
                                   network.rs (netsh)
```

## Diagnóstico (tela nova)

Módulo read-only que não passa pelo Serviço (exceto ponto de restauração e
DPC/ISR, que exigem elevação): serviços do Windows, apps de inicialização
(chaves Run), espaço/saúde de disco, discos físicos (SMART via
`Get-PhysicalDisk`), GPU e driver, plano de energia atual. Implementado
chamando `powershell.exe` com scripts fixos (nunca interpola entrada do
usuário) e parseando a saída como JSON — evita reimplementar WMI/Win32 à
mão em código não testável neste ambiente.

- **Exportar diagnóstico**: agrega tudo isso + CPU/RAM/processos num JSON em
  `Documentos\SystemForge-diagnostico-<timestamp>.json`.
- **Ponto de restauração real**: via `Checkpoint-Computer` (Serviço, precisa
  admin). O Windows limita a frequência (~1 por 24h) — esse erro chega
  como está, nunca é escondido.
- **Trace de DPC/ISR**: grava com `wpr.exe` (built-in, bem documentado — essa
  parte tem confiança alta). O resumo por driver via `xperf -a dpcisr` só
  roda se o Windows ADK estiver instalado, e o parser desse CSV **nunca foi
  validado contra uma saída real** (não há Windows disponível para gerar um
  trace de teste neste ambiente) — por isso a UI devolve os caminhos do
  `.etl`/`.csv` brutos para abrir no Windows Performance Analyzer, em vez de
  arriscar mostrar números processados por um parser não verificado.

## Limpeza de temporários

Só `%TEMP%` do usuário atual — nunca arquivo de sistema. Não passa pelo motor
de backup/restauração (não existe "desfazer" para arquivo apagado). Botão no
Dashboard, com confirmação.

## Catálogo e perfis

21 otimizações no catálogo seed: 5 nível 1 (seguras), 4 de segurança (HVCI,
VBS, SmartScreen, HAGS — nunca aplicáveis via perfil, só individualmente,
garantido por filtro no SQL de `apply_profile`/`restore_profile`, não só por
disciplina ao popular a migration) e 13 nível 2/3 reais (throttling de rede,
prioridade de CPU/GPU, SysMain/DiagTrack, tarefas de telemetria, planos de
energia). 6 perfis: padrão, Gaming competitivo, Gaming baixa latência,
Streaming, Trabalho e desenvolvimento, Economia de energia.

## O que falta para produção

- **Elevação/auto-start do Serviço.** Hoje é um processo manual (`cargo run -p
  systemforge-service` como admin). Falta o Core detectar que o Serviço não está
  rodando e disparar a instalação/início dele via UAC (`ShellExecuteExW` com verbo
  `runas`, ou registro como Windows Service via SCM) — documentado, não escondido.
- **Named pipe com ACL em vez de TCP loopback.** A porta 47732 hoje é só loopback
  + token aleatório por sessão, o que já impede acesso externo e replay entre
  reinícios do Serviço, mas um named pipe com ACL restrita a admins/LocalSystem é
  o endurecimento correto para produção.
- **Validação em máquina Windows real** de todo código `cfg(windows)` (Registro,
  SCM) e de todos os scripts PowerShell do módulo de diagnóstico — nunca
  rodaram de verdade, só foram conferidos linha a linha contra o código-fonte
  da crate `windows` e a documentação do PowerShell.
- **Parser de CSV do xperf** (DPC/ISR) não implementado de propósito — ver
  seção Diagnóstico acima.
- Uso de GPU (utilização/temperatura em tempo real) e latência de entrada na
  telemetria de jogos (hoje: CPU/RAM sempre reais, FPS real via PresentMon
  opcional, o resto "não disponível").
- Tela de Rede — o protocolo e o módulo do Serviço (`netsh`) já suportam
  `NetworkSetDns`/`NetworkSetDhcp`, mas não há UI nem tweak de catálogo
  usando ainda (exige selecionar o adaptador, não dá para automatizar via
  perfil sem entrada do usuário).
