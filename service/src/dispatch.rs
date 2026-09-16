use crate::modules::{network, power, registry, scheduled_tasks, services, startup};
use systemforge_shared::{ServiceRequest, ServiceResponse};

pub fn handle(req: ServiceRequest) -> ServiceResponse {
    match req {
        ServiceRequest::Auth { .. } => {
            ServiceResponse::Error("Auth é tratado pelo servidor antes do dispatch".into())
        }
        ServiceRequest::Ping => ServiceResponse::Pong,

        ServiceRequest::RegistryReadDword { path, value_name } => {
            match registry::read_dword(&path, &value_name) {
                Ok(v) => ServiceResponse::OptU32(v),
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::RegistryWriteDword { path, value_name, value } => {
            match registry::write_dword(&path, &value_name, value) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::RegistryDeleteValue { path, value_name } => {
            match registry::delete_value(&path, &value_name) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }

        ServiceRequest::PowerGetActiveScheme => match power::get_active_scheme() {
            Ok(guid) => ServiceResponse::OptString(Some(guid)),
            Err(e) => ServiceResponse::Error(e.to_string()),
        },
        ServiceRequest::PowerSetActiveScheme { scheme_guid } => {
            match power::set_active_scheme(&scheme_guid) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }

        ServiceRequest::StartupReadRunValue { location, app_name } => {
            match startup::read_run_value(&location, &app_name) {
                Ok(v) => ServiceResponse::OptString(v),
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::StartupDisable { location, app_name } => {
            match startup::disable_startup_item(&location, &app_name) {
                Ok(v) => ServiceResponse::OptString(v),
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::StartupRestore { location, app_name, previous_value } => {
            match startup::restore_startup_item(&location, &app_name, &previous_value) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }

        ServiceRequest::ServiceQueryStartType { service_name } => {
            match services::query_start_type(&service_name) {
                Ok(t) => ServiceResponse::OptString(Some(t.as_str().to_string())),
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::ServiceSetStartType { service_name, start_type } => {
            let Some(parsed) = services::StartType::parse(&start_type) else {
                return ServiceResponse::Error(format!("tipo de início desconhecido: {start_type}"));
            };
            match services::set_start_type(&service_name, parsed) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }

        ServiceRequest::ScheduledTaskQueryEnabled { task_path } => {
            match scheduled_tasks::query_enabled(&task_path) {
                Ok(v) => ServiceResponse::Bool(v),
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::ScheduledTaskSetEnabled { task_path, enabled } => {
            match scheduled_tasks::set_enabled(&task_path, enabled) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }

        ServiceRequest::NetworkGetDns { adapter_name } => match network::get_dns(&adapter_name) {
            Ok(list) => ServiceResponse::OptString(if list.is_empty() { None } else { Some(list.join(",")) }),
            Err(e) => ServiceResponse::Error(e.to_string()),
        },
        ServiceRequest::NetworkSetDns { adapter_name, primary, secondary } => {
            match network::set_dns_static(&adapter_name, &primary, secondary.as_deref()) {
                Ok(()) => ServiceResponse::Ok,
                Err(e) => ServiceResponse::Error(e.to_string()),
            }
        }
        ServiceRequest::NetworkSetDhcp { adapter_name } => match network::set_dns_dhcp(&adapter_name) {
            Ok(()) => ServiceResponse::Ok,
            Err(e) => ServiceResponse::Error(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong() {
        assert!(matches!(handle(ServiceRequest::Ping), ServiceResponse::Pong));
    }

    #[test]
    fn protected_service_is_refused_before_any_win32_call() {
        let resp = handle(ServiceRequest::ServiceSetStartType {
            service_name: "wuauserv".into(),
            start_type: "Disabled".into(),
        });
        match resp {
            ServiceResponse::Error(msg) => assert!(msg.contains("protegidos") || msg.contains("protected") || msg.to_lowercase().contains("protegido")),
            other => panic!("esperava Error, veio {other:?}"),
        }
    }

    #[test]
    fn unknown_start_type_is_rejected() {
        let resp = handle(ServiceRequest::ServiceSetStartType {
            service_name: "SysMain".into(),
            start_type: "Yolo".into(),
        });
        assert!(matches!(resp, ServiceResponse::Error(_)));
    }
}
