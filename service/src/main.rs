use systemforge_service::{server, transport};

/// Nesta fase, o binário roda como processo comum (não registrado no SCM).
/// Rodar com privilégio (LocalSystem/admin) é responsabilidade de quem
/// instala/inicia este processo — via Agendador de Tarefas com "executar com
/// privilégios mais altos" ou `sc create` apontando para este .exe. O
/// registro completo como Windows Service (ServiceMain/dispatcher table) é
/// trabalho de endurecimento de produção ainda não implementado; documentado
/// no README como próximo passo, não escondido.
fn main() {
    let app_data_dir = resolve_app_data_dir();
    let token = transport::generate_token();

    if let Err(e) = transport::write_token_file(&app_data_dir, &token) {
        eprintln!("systemforge-service: falha ao gravar o token em {app_data_dir:?}: {e}");
        std::process::exit(1);
    }

    println!("systemforge-service iniciado. Token gravado em {}", app_data_dir.join(systemforge_shared::TOKEN_FILE_NAME).display());

    if let Err(e) = server::run(token) {
        eprintln!("systemforge-service: falha ao iniciar o servidor: {e}");
        std::process::exit(1);
    }
}

/// Diretório do token de autenticação. Usa `%ProgramData%` (não `%APPDATA%`)
/// de propósito: é o mesmo em qualquer contexto de usuário/elevação, então
/// o Serviço (rodando elevado) e o Core (rodando como usuário comum) chegam
/// exatamente ao mesmo caminho sem precisar se coordenar por IPC prévio.
/// `src-tauri/src/modules/service_client.rs` replica esta mesma lógica —
/// qualquer mudança aqui precisa ser espelhada lá.
fn resolve_app_data_dir() -> std::path::PathBuf {
    if let Ok(program_data) = std::env::var("ProgramData") {
        std::path::PathBuf::from(program_data).join("SystemForgeOptimizer")
    } else {
        // Fora do Windows (dev/teste): usa um diretório local previsível.
        std::env::temp_dir().join("systemforge-optimizer-dev")
    }
}
