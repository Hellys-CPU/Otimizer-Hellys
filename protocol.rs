//! Protocolo de mensagens entre o Core e o Serviço privilegiado.
//!
//! Framing simples: 4 bytes de tamanho (little-endian) + payload JSON. Não
//! depende de nenhum transporte específico (named pipe no Windows, socket
//! Unix em testes) — só de `Read`/`Write`, então é testável em qualquer
//! plataforma com um par de streams em memória.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};

pub const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceRequest {
    /// Sempre a primeira mensagem de qualquer conexão. Sem isso (ou com o
    /// token errado), o Serviço fecha a conexão sem despachar mais nada.
    Auth { token: String },
    Ping,

    RegistryWriteDword { path: String, value_name: String, value: u32 },
    RegistryDeleteValue { path: String, value_name: String },
    RegistryReadDword { path: String, value_name: String },

    PowerGetActiveScheme,
    PowerSetActiveScheme { scheme_guid: String },

    StartupReadRunValue { location: String, app_name: String },
    StartupDisable { location: String, app_name: String },
    StartupRestore { location: String, app_name: String, previous_value: Option<String> },

    ServiceQueryStartType { service_name: String },
    ServiceSetStartType { service_name: String, start_type: String },

    ScheduledTaskQueryEnabled { task_path: String },
    ScheduledTaskSetEnabled { task_path: String, enabled: bool },

    NetworkGetDns { adapter_name: String },
    NetworkSetDns { adapter_name: String, primary: String, secondary: Option<String> },
    NetworkSetDhcp { adapter_name: String },

    /// Cria um ponto de restauração do Windows. O próprio Windows limita a
    /// frequência (1 por 24h por padrão) — esse erro chega como
    /// `ServiceResponse::Error` com a mensagem real do PowerShell, nunca é
    /// escondido ou fingido como sucesso.
    CreateRestorePoint { description: String },

    /// Grava um trace de kernel (DPC/ISR) com `wpr.exe` por `duration_secs`
    /// segundos. Exige elevação — por isso vive no Serviço, não no Core.
    CaptureDpcIsr { duration_secs: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceResponse {
    AuthOk,
    AuthRejected,
    Pong,
    Ok,
    OptU32(Option<u32>),
    OptString(Option<String>),
    Bool(bool),
    Error(String),
    /// `csv_path` só vem preenchido se `xperf.exe` (Windows ADK) estiver
    /// instalado — sem ele, só o `.etl` bruto é retornado (abrível no WPA).
    DpcIsrCapture { etl_path: String, csv_path: Option<String> },
}

pub fn write_message<W: Write, T: Serialize>(writer: &mut W, msg: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "mensagem excede o limite"));
    }
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

pub fn read_message<R: Read, T: DeserializeOwned>(reader: &mut R) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "mensagem excede o limite"));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrips_request_through_framing() {
        let req = ServiceRequest::RegistryWriteDword {
            path: "HKEY_CURRENT_USER\\Software\\Foo".into(),
            value_name: "Bar".into(),
            value: 42,
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &req).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: ServiceRequest = read_message(&mut cursor).unwrap();
        match decoded {
            ServiceRequest::RegistryWriteDword { path, value_name, value } => {
                assert_eq!(path, "HKEY_CURRENT_USER\\Software\\Foo");
                assert_eq!(value_name, "Bar");
                assert_eq!(value, 42);
            }
            _ => panic!("variante decodificada incorreta"),
        }
    }

    #[test]
    fn rejects_truncated_stream() {
        let mut cursor = Cursor::new(vec![10, 0, 0, 0, 1, 2]); // diz 10 bytes, só tem 2
        let result: io::Result<ServiceResponse> = read_message(&mut cursor);
        assert!(result.is_err());
    }
}
