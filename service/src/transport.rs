//! Transporte da conexão Core <-> Serviço: TCP em loopback (127.0.0.1),
//! nunca exposto externamente, autenticado por um token aleatório gerado a
//! cada início do Serviço e compartilhado com o Core via um arquivo no
//! diretório de dados do app.
//!
//! TCP loopback foi escolhido em vez de named pipe nativo porque: (a) o
//! protocolo em si (framing + allowlist) fica 100% testável em qualquer
//! plataforma, inclusive fora do Windows; (b) evita código Win32 não
//! testável escrito à mão para I/O de pipe. Endurecimento futuro (named
//! pipe com ACL restrita a admins/LocalSystem) é um passo de produção, não
//! um requisito para a lógica de negócio estar correta.

use rand::Rng;
use std::fs;
use std::io;
use std::net::TcpListener;
use std::path::Path;

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| format!("{:x}", rng.gen_range(0..16))).collect()
}

pub fn write_token_file(app_data_dir: &Path, token: &str) -> io::Result<()> {
    fs::create_dir_all(app_data_dir)?;
    let path = app_data_dir.join(systemforge_shared::TOKEN_FILE_NAME);
    fs::write(path, token)
}

pub fn bind() -> io::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", systemforge_shared::IPC_PORT))
}
