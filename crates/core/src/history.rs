// SPDX-License-Identifier: GPL-3.0-or-later
//! Histórico de apuração persistido em JSON (escrita atômica).

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::model::{PontoHistorico, Snapshot};

pub const MAX_PONTOS: usize = 5000;

#[derive(Debug, Clone, Default)]
pub struct History {
    path: Option<PathBuf>,
    pontos: Vec<PontoHistorico>,
}

impl History {
    /// Histórico só em memória.
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// Carrega de `path` (arquivo ausente ou corrompido => vazio; nunca impede o app de abrir).
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let pontos = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Self {
            path: Some(path),
            pontos,
        }
    }

    pub fn pontos(&self) -> &[PontoHistorico] {
        &self.pontos
    }

    /// Acrescenta o ponto do snapshot se houve mudança em relação ao último. Retorna se acrescentou.
    pub fn record(&mut self, snap: &Snapshot) -> Result<bool> {
        let p = PontoHistorico::from_snapshot(snap);
        if self.pontos.last().is_some_and(|l| {
            l.candidatos == p.candidatos && l.percentual_apurado == p.percentual_apurado
        }) {
            return Ok(false);
        }
        self.pontos.push(p);
        if self.pontos.len() > MAX_PONTOS {
            let excess = self.pontos.len() - MAX_PONTOS;
            self.pontos.drain(..excess);
        }
        self.save()?;
        Ok(true)
    }

    fn save(&self) -> Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        write_atomic(path, &serde_json::to_vec(&self.pontos)?)
    }
}

/// Escreve em arquivo temporário e renomeia.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
