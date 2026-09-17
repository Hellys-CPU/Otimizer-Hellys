//! Inventário e diagnóstico — tudo aqui é só leitura, não exige elevação e
//! não passa pelo Serviço privilegiado.
//!
//! Implementado chamando o PowerShell (built-in em qualquer Windows 10/11)
//! com um script fixo por função — nunca interpolando entrada do usuário no
//! script, então não há superfície de injeção. Preferido a reimplementar
//! WMI/Win32 à mão em Rust: PowerShell já expõe `Get-Service`,
//! `Get-Volume`, `Get-PhysicalDisk` e `Get-CimInstance` de forma estável, e
//! isso reduz a quantidade de código Win32 não testável neste ambiente.
//!
//! Duas armadilhas do PowerShell evitadas de propósito em todo script aqui:
//! 1. Enums do .NET (Status, StartType, HealthStatus...) viram número no
//!    JSON a menos que se chame `.ToString()` explicitamente.
//! 2. `objeto | ConvertTo-Json` "desembrulha" um array de 1 item em objeto
//!    solto (isso acontece em QUALQUER versão do PowerShell, é o
//!    comportamento do pipeline, não um bug do cmdlet) — por isso sempre
//!    passamos a coleção via `-InputObject`, nunca por pipe, garantindo
//!    JSON array mesmo com 0 ou 1 resultado. `-AsArray` (que resolveria
//!    isso de outro jeito) só existe no PowerShell 6.2+, e `powershell.exe`
//!    no Windows 10/11 é a 5.1 por padrão — não dá pra depender dele.

use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiagnosticsError {
    #[error("falha ao executar powershell: {0}")]
    CommandFailed(String),
    #[error("powershell retornou código de erro: {0}")]
    NonZeroExit(i32),
    #[error("não foi possível interpretar a saída do powershell: {0}")]
    ParseError(String),
}

fn run_powershell_json<T: serde::de::DeserializeOwned>(script: &str) -> Result<Vec<T>, DiagnosticsError> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| DiagnosticsError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(DiagnosticsError::NonZeroExit(output.status.code().unwrap_or(-1)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(trimmed).map_err(|e| DiagnosticsError::ParseError(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "StartType")]
    pub start_type: String,
}

pub fn list_services() -> Result<Vec<ServiceInfo>, DiagnosticsError> {
    run_powershell_json(
        "$r = @(Get-Service | Select-Object Name,DisplayName,\
         @{N='Status';E={$_.Status.ToString()}},@{N='StartType';E={$_.StartType.ToString()}}); \
         ConvertTo-Json -InputObject $r -Depth 2",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    #[serde(rename = "DriveLetter")]
    pub drive_letter: Option<String>,
    #[serde(rename = "FileSystemLabel")]
    pub label: Option<String>,
    #[serde(rename = "SizeGb")]
    pub size_gb: Option<f64>,
    #[serde(rename = "FreeGb")]
    pub free_gb: Option<f64>,
    #[serde(rename = "HealthStatus")]
    pub health_status: Option<String>,
}

pub fn list_disks() -> Result<Vec<DiskInfo>, DiagnosticsError> {
    run_powershell_json(
        "$r = @(Get-Volume | Where-Object { $_.DriveLetter } | Select-Object DriveLetter,FileSystemLabel,\
         @{N='SizeGb';E={[math]::Round($_.Size/1GB,1)}},@{N='FreeGb';E={[math]::Round($_.SizeRemaining/1GB,1)}},\
         @{N='HealthStatus';E={$_.HealthStatus.ToString()}}); \
         ConvertTo-Json -InputObject $r -Depth 2",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalDiskInfo {
    #[serde(rename = "FriendlyName")]
    pub friendly_name: String,
    #[serde(rename = "MediaType")]
    pub media_type: String,
    #[serde(rename = "HealthStatus")]
    pub health_status: String,
    #[serde(rename = "OperationalStatus")]
    pub operational_status: String,
}

pub fn list_physical_disks() -> Result<Vec<PhysicalDiskInfo>, DiagnosticsError> {
    run_powershell_json(
        "$r = @(Get-PhysicalDisk | Select-Object FriendlyName,@{N='MediaType';E={$_.MediaType.ToString()}},\
         @{N='HealthStatus';E={$_.HealthStatus.ToString()}},@{N='OperationalStatus';E={$_.OperationalStatus.ToString()}}); \
         ConvertTo-Json -InputObject $r -Depth 2",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "DriverVersion")]
    pub driver_version: Option<String>,
    #[serde(rename = "AdapterRAM")]
    pub adapter_ram: Option<i64>,
}

pub fn list_gpus() -> Result<Vec<GpuInfo>, DiagnosticsError> {
    run_powershell_json(
        "$r = @(Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM); \
         ConvertTo-Json -InputObject $r -Depth 2",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverInfo {
    #[serde(rename = "DeviceName")]
    pub device_name: Option<String>,
    #[serde(rename = "DriverVersion")]
    pub driver_version: Option<String>,
    #[serde(rename = "DriverDate")]
    pub driver_date: Option<String>,
    #[serde(rename = "IsSigned")]
    pub is_signed: Option<bool>,
}

/// Lista drivers de dispositivo (não drivers de sistema/kernel genéricos).
/// `driver_date` é usado pela UI só como sinal — "driver com mais de X anos"
/// é heurística, não indica defeito por si só.
pub fn list_drivers() -> Result<Vec<DriverInfo>, DiagnosticsError> {
    run_powershell_json(
        "$r = @(Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceName } | \
         Select-Object DeviceName,DriverVersion,\
         @{N='DriverDate';E={ if ($_.DriverDate) { $_.DriverDate.ToString('yyyy-MM-dd') } else { $null } }},\
         IsSigned); \
         ConvertTo-Json -InputObject $r -Depth 2",
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupAppInfo {
    pub name: String,
    pub command: String,
    pub location: String,
}

/// Lê as chaves Run do Registro (HKCU e HKLM) — leitura, não precisa do
/// Serviço. Não lista a pasta de Startup (atalhos .lnk) nesta primeira
/// versão para manter o escopo simples; Run/RunOnce já cobre a maioria dos
/// apps que se auto-registram para iniciar com o Windows.
pub fn list_startup_apps() -> Result<Vec<StartupAppInfo>, DiagnosticsError> {
    let script = r#"
        $items = @()
        $paths = @(
            @{Path='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'; Location='HKCU Run'},
            @{Path='HKLM:\Software\Microsoft\Windows\CurrentVersion\Run'; Location='HKLM Run'}
        )
        foreach ($p in $paths) {
            if (Test-Path $p.Path) {
                $props = Get-ItemProperty -Path $p.Path
                foreach ($prop in $props.PSObject.Properties) {
                    if ($prop.Name -notmatch '^PS') {
                        $items += [PSCustomObject]@{ name = $prop.Name; command = "$($prop.Value)"; location = $p.Location }
                    }
                }
            }
        }
        ConvertTo-Json -InputObject @($items) -Depth 2
    "#;
    run_powershell_json(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_typical_service_json_array() {
        let json = r#"[{"Name":"SysMain","DisplayName":"Superfetch","Status":"Running","StartType":"Automatic"}]"#;
        let parsed: Vec<ServiceInfo> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].status, "Running");
    }

    #[test]
    fn deserializes_empty_array() {
        let parsed: Vec<ServiceInfo> = serde_json::from_str("[]").unwrap();
        assert!(parsed.is_empty());
    }
}
