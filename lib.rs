pub mod action;
pub mod protocol;
pub mod validation;

pub use action::{ActionError, ActionId};
pub use protocol::{ServiceRequest, ServiceResponse};
pub use validation::{ValidationError, PROTECTED_SERVICES};

/// Porta local (loopback apenas) em que o Serviço privilegiado escuta.
/// Não é exposta externamente — o Core só se conecta a 127.0.0.1.
pub const IPC_PORT: u16 = 47732;

/// GUID do esquema de energia "Alto Desempenho" embutido do Windows —
/// conhecido e documentado pela Microsoft, usado tanto pelo Core (para saber
/// qual esquema pedir) quanto pelo Serviço (para validar o pedido).
pub const SCHEME_HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";

/// Nome do arquivo, dentro do diretório de dados do app, onde o Serviço
/// grava um token aleatório gerado a cada início. O Core lê o mesmo arquivo
/// e envia o token como primeira mensagem da conexão — sem o token correto,
/// o Serviço encerra a conexão sem executar nenhuma ação.
pub const TOKEN_FILE_NAME: &str = "service.token";
