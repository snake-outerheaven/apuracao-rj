// SPDX-License-Identifier: GPL-3.0-or-later
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("falha de rede: {0}")]
    Http(#[from] reqwest::Error),
    #[error("HTTP {status} em {url}")]
    Status { status: u16, url: String },
    #[error("JSON inválido: {0}")]
    Json(#[from] serde_json::Error),
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("dados inesperados: {0}")]
    Parse(String),
    #[error("configuração inválida: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
