pub mod backup;
pub mod game_watcher;
pub mod profile;
pub mod telemetry;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("banco de dados: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("registro: {0}")]
    Registry(#[from] crate::modules::registry::RegistryError),
    #[error("ação não permitida: {0}")]
    Action(#[from] systemforge_shared::ActionError),
    #[error("serviço privilegiado: {0}")]
    Service(String),
    #[error("perfil não encontrado: {0}")]
    ProfileNotFound(String),
    #[error("otimização não encontrada: {0}")]
    OptimizationNotFound(String),
    #[error("esta otimização exige seleção manual de item e não pode ser aplicada automaticamente pelo perfil: {0}")]
    RequiresManualSelection(String),
    #[error("nenhum snapshot aplicável encontrado para restauração de {0}")]
    NoSnapshotToRestore(String),
}
