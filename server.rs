use std::io;
use std::net::{TcpListener, TcpStream};
use systemforge_shared::protocol::{read_message, write_message};
use systemforge_shared::{ServiceRequest, ServiceResponse};

use crate::dispatch;

/// Loop principal de produção: escuta na porta fixa do protocolo.
pub fn run(expected_token: String) -> io::Result<()> {
    let listener = crate::transport::bind()?;
    run_on(listener, expected_token)
}

/// Aceita conexões em um listener já vinculado (produção usa a porta fixa;
/// testes usam uma porta efêmera para poder rodar em paralelo sem conflito).
pub fn run_on(listener: TcpListener, expected_token: String) -> io::Result<()> {
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };
        let token = expected_token.clone();
        std::thread::spawn(move || {
            let _ = handle_connection(stream, &token);
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, expected_token: &str) -> io::Result<()> {
    let first: ServiceRequest = read_message(&mut stream)?;
    match first {
        ServiceRequest::Auth { token } if token == expected_token => {
            write_message(&mut stream, &ServiceResponse::AuthOk)?;
        }
        _ => {
            write_message(&mut stream, &ServiceResponse::AuthRejected)?;
            return Ok(());
        }
    }

    loop {
        let req: ServiceRequest = match read_message(&mut stream) {
            Ok(r) => r,
            Err(_) => return Ok(()), // cliente desconectou ou stream corrompido
        };
        let resp = dispatch::handle(req);
        write_message(&mut stream, &resp)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;

    fn spawn_test_server(token: &str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let token = token.to_string();
        std::thread::spawn(move || {
            let _ = run_on(listener, token);
        });
        port
    }

    #[test]
    fn rejects_wrong_token() {
        let port = spawn_test_server("correct-token");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write_message(&mut stream, &ServiceRequest::Auth { token: "wrong".into() }).unwrap();
        let resp: ServiceResponse = read_message(&mut stream).unwrap();
        assert!(matches!(resp, ServiceResponse::AuthRejected));
    }

    #[test]
    fn accepts_correct_token_and_answers_ping() {
        let port = spawn_test_server("the-real-token");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write_message(&mut stream, &ServiceRequest::Auth { token: "the-real-token".into() }).unwrap();
        let auth_resp: ServiceResponse = read_message(&mut stream).unwrap();
        assert!(matches!(auth_resp, ServiceResponse::AuthOk));

        write_message(&mut stream, &ServiceRequest::Ping).unwrap();
        let resp: ServiceResponse = read_message(&mut stream).unwrap();
        assert!(matches!(resp, ServiceResponse::Pong));
    }

    #[test]
    fn protected_service_change_is_rejected_end_to_end() {
        let port = spawn_test_server("tok");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write_message(&mut stream, &ServiceRequest::Auth { token: "tok".into() }).unwrap();
        let _: ServiceResponse = read_message(&mut stream).unwrap();

        write_message(
            &mut stream,
            &ServiceRequest::ServiceSetStartType {
                service_name: "WinDefend".into(),
                start_type: "Disabled".into(),
            },
        )
        .unwrap();
        let resp: ServiceResponse = read_message(&mut stream).unwrap();
        assert!(matches!(resp, ServiceResponse::Error(_)));
    }

    #[test]
    fn unauthenticated_request_is_never_dispatched() {
        // Manda um Ping como primeira mensagem (sem Auth) — o servidor deve
        // rejeitar e fechar, nunca despachar a ação.
        let port = spawn_test_server("tok");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write_message(&mut stream, &ServiceRequest::Ping).unwrap();
        let resp: ServiceResponse = read_message(&mut stream).unwrap();
        assert!(matches!(resp, ServiceResponse::AuthRejected));
    }
}
