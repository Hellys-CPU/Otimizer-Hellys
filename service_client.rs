//! Cliente do Serviço privilegiado. Esta é a ÚNICA forma pela qual o Core
//! pede uma escrita no Registro/Serviços/Tarefas/Rede/Energia — o Core nunca
//! chama Win32 diretamente para essas operações.
//!
//! Autentica com o token gravado por `systemforge-service` em
//! `%ProgramData%\SystemForgeOptimizer\service.token` (mesmo caminho
//! calculado independentemente pelos dois processos — ver
//! `service/src/main.rs::resolve_app_data_dir`, que precisa continuar
//! espelhando esta função se um dos dois mudar).

use crate::modules::service_launcher;
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;
use systemforge_shared::protocol::{read_message, write_message};
use systemforge_shared::{ServiceRequest, ServiceResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceClientError {
    #[error("não foi possível ler o token do Serviço em {0}: {1}. O Serviço privilegiado está rodando?")]
    TokenUnavailable(String, String),
    #[error("falha de conexão com o Serviço privilegiado: {0}")]
    Connection(String),
    #[error("autenticação com o Serviço privilegiado foi rejeitada — token desatualizado ou Serviço reiniciou")]
    AuthRejected,
    #[error("resposta inesperada do Serviço: {0:?}")]
    UnexpectedResponse(ServiceResponse),
    #[error("o Serviço reportou um erro: {0}")]
    ServiceError(String),
    #[error("não foi possível iniciar o Serviço privilegiado automaticamente: {0}")]
    AutoStartFailed(String),
}

fn resolve_app_data_dir() -> PathBuf {
    if let Ok(program_data) = std::env::var("ProgramData") {
        PathBuf::from(program_data).join("SystemForgeOptimizer")
    } else {
        std::env::temp_dir().join("systemforge-optimizer-dev")
    }
}

fn read_token() -> Result<String, ServiceClientError> {
    let path = resolve_app_data_dir().join(systemforge_shared::TOKEN_FILE_NAME);
    std::fs::read_to_string(&path)
        .map_err(|e| ServiceClientError::TokenUnavailable(path.display().to_string(), e.to_string()))
}

fn is_service_down(err: &ServiceClientError) -> bool {
    matches!(
        err,
        ServiceClientError::TokenUnavailable(..) | ServiceClientError::Connection(_)
    )
}

/// Tenta o pedido; se o Serviço parecer não estar rodando (token ausente ou
/// conexão recusada), dispara a elevação via UAC uma vez e tenta de novo
/// depois de esperar o token aparecer. Isso substitui o antigo fluxo manual
/// (usuário abrindo um terminal Admin e rodando `systemforge-service.exe`
/// à mão) — a causa mais comum de "clico em Aplicar e não acontece nada".
fn send_request(req: ServiceRequest) -> Result<ServiceResponse, ServiceClientError> {
    match try_send_request(&req) {
        Err(err) if is_service_down(&err) => {
            service_launcher::ensure_launched().map_err(|e| ServiceClientError::AutoStartFailed(e.to_string()))?;
            if !service_launcher::wait_for_token(Duration::from_secs(8)) {
                return Err(ServiceClientError::AutoStartFailed(
                    "o Serviço não terminou de iniciar a tempo".to_string(),
                ));
            }
            // O token pode existir um instante antes do listener TCP estar
            // de fato aceitando conexões — pequena folga para evitar corrida.
            std::thread::sleep(Duration::from_millis(300));
            try_send_request(&req)
        }
        other => other,
    }
}

fn try_send_request(req: &ServiceRequest) -> Result<ServiceResponse, ServiceClientError> {
    let token = read_token()?;
    let mut stream = TcpStream::connect(("127.0.0.1", systemforge_shared::IPC_PORT))
        .map_err(|e| ServiceClientError::Connection(e.to_string()))?;

    write_message(&mut stream, &ServiceRequest::Auth { token })
        .map_err(|e| ServiceClientError::Connection(e.to_string()))?;
    let auth_resp: ServiceResponse =
        read_message(&mut stream).map_err(|e| ServiceClientError::Connection(e.to_string()))?;
    match auth_resp {
        ServiceResponse::AuthOk => {}
        ServiceResponse::AuthRejected => return Err(ServiceClientError::AuthRejected),
        other => return Err(ServiceClientError::UnexpectedResponse(other)),
    }

    write_message(&mut stream, req).map_err(|e| ServiceClientError::Connection(e.to_string()))?;
    read_message(&mut stream).map_err(|e| ServiceClientError::Connection(e.to_string()))
}

fn expect_ok(resp: ServiceResponse) -> Result<(), ServiceClientError> {
    match resp {
        ServiceResponse::Ok => Ok(()),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}

fn expect_opt_u32(resp: ServiceResponse) -> Result<Option<u32>, ServiceClientError> {
    match resp {
        ServiceResponse::OptU32(v) => Ok(v),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}

fn expect_opt_string(resp: ServiceResponse) -> Result<Option<String>, ServiceClientError> {
    match resp {
        ServiceResponse::OptString(v) => Ok(v),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}

fn expect_bool(resp: ServiceResponse) -> Result<bool, ServiceClientError> {
    match resp {
        ServiceResponse::Bool(v) => Ok(v),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}

pub fn registry_read_dword(path: &str, value_name: &str) -> Result<Option<u32>, ServiceClientError> {
    let resp = send_request(ServiceRequest::RegistryReadDword {
        path: path.to_string(),
        value_name: value_name.to_string(),
    })?;
    expect_opt_u32(resp)
}

pub fn registry_write_dword(path: &str, value_name: &str, value: u32) -> Result<(), ServiceClientError> {
    let resp = send_request(ServiceRequest::RegistryWriteDword {
        path: path.to_string(),
        value_name: value_name.to_string(),
        value,
    })?;
    expect_ok(resp)
}

pub fn registry_delete_value(path: &str, value_name: &str) -> Result<(), ServiceClientError> {
    let resp = send_request(ServiceRequest::RegistryDeleteValue {
        path: path.to_string(),
        value_name: value_name.to_string(),
    })?;
    expect_ok(resp)
}

pub fn power_get_active_scheme() -> Result<Option<String>, ServiceClientError> {
    expect_opt_string(send_request(ServiceRequest::PowerGetActiveScheme)?)
}

pub fn power_set_active_scheme(scheme_guid: &str) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::PowerSetActiveScheme { scheme_guid: scheme_guid.to_string() })?)
}

pub fn startup_disable(location: &str, app_name: &str) -> Result<Option<String>, ServiceClientError> {
    expect_opt_string(send_request(ServiceRequest::StartupDisable {
        location: location.to_string(),
        app_name: app_name.to_string(),
    })?)
}

pub fn startup_restore(location: &str, app_name: &str, previous_value: Option<String>) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::StartupRestore {
        location: location.to_string(),
        app_name: app_name.to_string(),
        previous_value,
    })?)
}

pub fn service_query_start_type(service_name: &str) -> Result<Option<String>, ServiceClientError> {
    expect_opt_string(send_request(ServiceRequest::ServiceQueryStartType { service_name: service_name.to_string() })?)
}

pub fn service_set_start_type(service_name: &str, start_type: &str) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::ServiceSetStartType {
        service_name: service_name.to_string(),
        start_type: start_type.to_string(),
    })?)
}

pub fn scheduled_task_query_enabled(task_path: &str) -> Result<bool, ServiceClientError> {
    expect_bool(send_request(ServiceRequest::ScheduledTaskQueryEnabled { task_path: task_path.to_string() })?)
}

pub fn scheduled_task_set_enabled(task_path: &str, enabled: bool) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::ScheduledTaskSetEnabled { task_path: task_path.to_string(), enabled })?)
}

pub fn network_set_dns(adapter_name: &str, primary: &str, secondary: Option<&str>) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::NetworkSetDns {
        adapter_name: adapter_name.to_string(),
        primary: primary.to_string(),
        secondary: secondary.map(String::from),
    })?)
}

pub fn network_set_dhcp(adapter_name: &str) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::NetworkSetDhcp { adapter_name: adapter_name.to_string() })?)
}

pub fn create_restore_point(description: &str) -> Result<(), ServiceClientError> {
    expect_ok(send_request(ServiceRequest::CreateRestorePoint { description: description.to_string() })?)
}

pub struct DpcIsrCapture {
    pub etl_path: String,
    pub csv_path: Option<String>,
}

/// Bloqueia pela duração da captura (não é instantâneo — está gravando um
/// trace de kernel de verdade). Chame de uma thread/task que não trave a UI.
pub fn capture_dpc_isr(duration_secs: u32) -> Result<DpcIsrCapture, ServiceClientError> {
    match send_request(ServiceRequest::CaptureDpcIsr { duration_secs })? {
        ServiceResponse::DpcIsrCapture { etl_path, csv_path } => Ok(DpcIsrCapture { etl_path, csv_path }),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}

pub fn ping() -> Result<(), ServiceClientError> {
    expect_ok_pong(send_request(ServiceRequest::Ping)?)
}

fn expect_ok_pong(resp: ServiceResponse) -> Result<(), ServiceClientError> {
    match resp {
        ServiceResponse::Pong => Ok(()),
        ServiceResponse::Error(msg) => Err(ServiceClientError::ServiceError(msg)),
        other => Err(ServiceClientError::UnexpectedResponse(other)),
    }
}
