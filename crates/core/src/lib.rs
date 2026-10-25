// SPDX-License-Identifier: GPL-3.0-or-later
//! `apuracao-core`: toda a lógica do apuracao-rj (modelos, parser do TSE, coleta, histórico, mock).
//! O `src-tauri` é só uma casca sobre este crate.

pub mod endpoints;
pub mod error;
pub mod fetcher;
pub mod history;
pub mod mock;
pub mod model;
pub mod parser;
pub mod service;

pub use error::{Error, Result};
pub use model::{Abrangencia, Candidato, Fonte, PontoHistorico, Settings, Snapshot, Totais};
