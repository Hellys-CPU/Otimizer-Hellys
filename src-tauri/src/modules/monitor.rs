//! Monitor de hardware — CPU, RAM e processos de maior consumo.
//!
//! Usa `sysinfo`, que é multiplataforma (funciona também fora do Windows),
//! diferente dos demais módulos deste diretório que só existem em `cfg(windows)`.

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub top_processes: Vec<ProcessInfo>,
}

/// Tira uma foto do estado atual do sistema. A primeira leitura de CPU do
/// `sysinfo` é sempre 0% — por isso são feitas duas leituras separadas pelo
/// intervalo mínimo recomendado pela própria crate.
pub fn snapshot(top_n: usize) -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let cpu_usage = sys.global_cpu_usage();
    let ram_total_mb = sys.total_memory() / 1024 / 1024;
    let ram_used_mb = sys.used_memory() / 1024 / 1024;
    let ram_usage_percent = if ram_total_mb > 0 {
        (ram_used_mb as f32 / ram_total_mb as f32) * 100.0
    } else {
        0.0
    };

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() / 1024 / 1024,
        })
        .collect();
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(top_n);

    SystemSnapshot {
        cpu_usage,
        ram_usage_percent,
        ram_used_mb,
        ram_total_mb,
        top_processes: processes,
    }
}

/// Enumera nomes de processos em execução — usado pelo detector de jogos.
pub fn running_process_names() -> Vec<(u32, String)> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    sys.processes()
        .values()
        .map(|p| (p.pid().as_u32(), p.name().to_string_lossy().to_string()))
        .collect()
}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}
