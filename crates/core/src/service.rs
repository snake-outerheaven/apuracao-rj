// SPDX-License-Identifier: GPL-3.0-or-later
//! Orquestra provider (TSE ou mock), cache do último snapshot, histórico e configurações.
//! É o que o `src-tauri` usa; toda a lógica testável fica aqui.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::endpoints::{parse_municipios, Endpoints};
use crate::error::{Error, Result};
use crate::fetcher::TseProvider;
use crate::history::{write_atomic, History};
use crate::mock::MockProvider;
use crate::model::{PontoHistorico, Settings, Snapshot};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Diretório de dados do app (histórico, configurações).
    pub data_dir: PathBuf,
    /// Caminho de `config/endpoints.json`.
    pub endpoints_path: PathBuf,
    /// Caminho de `config/mun-e006258-cm.json` (opcional; se ausente, baixa da URL da config).
    pub municipios_path: PathBuf,
    /// Caminho de `fixtures/mock`.
    pub mock_dir: PathBuf,
}

struct Inner {
    settings: Settings,
    snapshot: Option<Snapshot>,
    history: History,
    tse: Option<TseProvider>,
    mock: Option<MockProvider>,
}

pub struct Service {
    cfg: ServiceConfig,
    inner: Mutex<Inner>,
    /// Serializa as buscas (evita ciclos concorrentes).
    refresh_lock: tokio::sync::Mutex<()>,
}

fn history_path(cfg: &ServiceConfig, mock: bool) -> PathBuf {
    cfg.data_dir.join(if mock {
        "historico-mock.json"
    } else {
        "historico.json"
    })
}

impl Service {
    pub fn new(cfg: ServiceConfig) -> Self {
        let settings = std::fs::read_to_string(cfg.data_dir.join("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Settings>(&t).ok())
            .unwrap_or_default()
            .sanitized();
        let history = History::load(history_path(&cfg, settings.mock));
        Self {
            inner: Mutex::new(Inner {
                settings,
                snapshot: None,
                history,
                tse: None,
                mock: None,
            }),
            cfg,
            refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn settings(&self) -> Settings {
        self.lock().settings.clone()
    }

    pub fn set_settings(&self, new: Settings) -> Result<Settings> {
        let new = new.sanitized();
        let mut g = self.lock();
        if g.settings.mock != new.mock {
            // Fonte diferente: descarta cache e usa o histórico próprio da fonte.
            g.snapshot = None;
            g.mock = None;
            g.history = History::load(history_path(&self.cfg, new.mock));
        }
        g.settings = new.clone();
        drop(g);
        write_atomic(
            &self.cfg.data_dir.join("settings.json"),
            &serde_json::to_vec(&new)?,
        )?;
        Ok(new)
    }

    pub fn history(&self) -> Vec<PontoHistorico> {
        self.lock().history.pontos().to_vec()
    }

    /// Último snapshot em cache; se não houver, busca.
    pub async fn get_snapshot(&self) -> Result<Snapshot> {
        if let Some(s) = self.lock().snapshot.clone() {
            return Ok(s);
        }
        self.refresh().await
    }

    /// Força busca. Em falha, o cache (se houver) fica marcado `desatualizado` e o erro é devolvido.
    pub async fn refresh(&self) -> Result<Snapshot> {
        let _guard = self.refresh_lock.lock().await;
        let result = self.fetch_once().await;
        let mut g = self.lock();
        match result {
            Ok(snap) => {
                // Falha ao gravar histórico não invalida o dado coletado.
                let _ = g.history.record(&snap);
                g.snapshot = Some(snap.clone());
                Ok(snap)
            }
            Err(e) => {
                if let Some(s) = g.snapshot.as_mut() {
                    s.desatualizado = true;
                }
                Err(e)
            }
        }
    }

    /// Último snapshot em cache sem buscar (com a flag `desatualizado` atual).
    pub fn cached(&self) -> Option<Snapshot> {
        self.lock().snapshot.clone()
    }

    async fn fetch_once(&self) -> Result<Snapshot> {
        let (is_mock, tse) = {
            let mut g = self.lock();
            let is_mock = g.settings.mock;
            if is_mock && g.mock.is_none() {
                g.mock = Some(MockProvider::from_dir(&self.cfg.mock_dir)?);
            }
            if !is_mock && g.tse.is_none() {
                let mut ep = Endpoints::from_file(&self.cfg.endpoints_path)?;
                if let Ok(t) = std::fs::read_to_string(&self.cfg.municipios_path) {
                    ep.municipios = parse_municipios(&t, &ep.uf)?;
                }
                g.tse = Some(TseProvider::new(ep)?);
            }
            (is_mock, g.tse.clone())
        };
        if is_mock {
            let mut g = self.lock();
            let m = g
                .mock
                .as_mut()
                .ok_or_else(|| Error::Config("mock indisponível".into()))?;
            Ok(m.avancar())
        } else {
            tse.ok_or_else(|| Error::Config("provider TSE indisponível".into()))?
                .fetch()
                .await
        }
    }
}
