//! Limpeza de arquivos temporários do usuário — única "limpeza" que o app
//! faz, e deliberadamente restrita a `%TEMP%` (que já é, por definição,
//! espaço de cache descartável do próprio usuário — nunca arquivo de
//! sistema, nunca fora dessa árvore de diretórios).
//!
//! Não roda pelo Serviço privilegiado: apagar o próprio temp do usuário não
//! exige elevação, e arquivos travados (em uso por outro processo) apenas
//! falham silenciosamente e são contados em `errors` — igual ao
//! comportamento do Liberador de Espaço em Disco do Windows.
//!
//! Não é reversível e por isso nunca passa pelo motor de backup/restauração
//! dos outros tweaks — é uma ação de mão única, com seu próprio comando.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupReport {
    pub files_deleted: u64,
    pub bytes_freed: u64,
    pub errors: u64,
}

pub fn clean_user_temp() -> CleanupReport {
    let mut report = CleanupReport::default();
    clean_dir_contents(&std::env::temp_dir(), &mut report);
    report
}

fn clean_dir_contents(dir: &Path, report: &mut CleanupReport) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            report.errors += 1;
            continue;
        };
        if metadata.is_dir() {
            clean_dir_contents(&path, report);
            let _ = fs::remove_dir(&path); // só some se já estiver vazio
        } else {
            let size = metadata.len();
            match fs::remove_file(&path) {
                Ok(()) => {
                    report.files_deleted += 1;
                    report.bytes_freed += size;
                }
                Err(_) => report.errors += 1, // arquivo em uso/sem permissão — ignora e segue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_only_the_given_directory_tree() {
        let base = std::env::temp_dir().join(format!("sfo-cleanup-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(base.join("sub")).unwrap();
        fs::write(base.join("a.tmp"), b"1234567890").unwrap();
        fs::write(base.join("sub").join("b.tmp"), b"12345").unwrap();

        let mut report = CleanupReport::default();
        clean_dir_contents(&base, &mut report);

        assert_eq!(report.files_deleted, 2);
        assert_eq!(report.bytes_freed, 15);
        assert!(!base.join("a.tmp").exists());
        assert!(!base.join("sub").exists(), "subpasta vazia deve ser removida também");
        assert!(base.exists(), "o diretório raiz passado nunca é removido, só seu conteúdo");

        fs::remove_dir_all(&base).ok();
    }
}
