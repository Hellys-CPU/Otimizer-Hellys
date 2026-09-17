use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub nome: String,
    pub descricao: String,
    pub icone: String,
    pub cor: String,
    pub nivel_agressividade: i64,
    pub otimizacoes_ids: Vec<String>,
    pub otimizacoes_proibidas_ids: Vec<String>,
    pub requer_administrador: bool,
    pub ultima_aplicacao: Option<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Optimization {
    pub id: String,
    pub nome: String,
    pub categoria: String,
    pub descricao: String,
    pub beneficio_esperado: String,
    pub risco: String,
    pub requer_reinicializacao: bool,
    pub requer_administrador: bool,
    pub valor_atual: Option<String>,
    pub valor_recomendado: Option<String>,
    pub fonte_tecnica: String,
    pub ativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSession {
    pub id: String,
    pub profile_id: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLogEntry {
    pub id: String,
    pub session_id: String,
    pub command: String,
    pub user: String,
    pub timestamp: String,
    pub result: String,
    pub error_message: Option<String>,
    pub return_code: Option<i64>,
    pub requires_reboot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub ram_usage: f64,
    pub windows_build: String,
    pub is_admin: bool,
}
