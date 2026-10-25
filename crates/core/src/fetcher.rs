// SPDX-License-Identifier: GPL-3.0-or-later
//! Coleta HTTP assíncrona: ETag/If-None-Match, retry com backoff exponencial e timeout.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::endpoints::{parse_municipios, Endpoints, MunicipioRef};
use crate::error::{Error, Result};
use crate::model::{Fonte, Snapshot};
use crate::parser::{build_snapshot, parse_abrangencia, Parsed};

#[derive(Debug, Clone)]
struct CacheEntry {
    etag: Option<String>,
    body: Arc<String>,
}

#[derive(Debug, Clone)]
pub struct Fetcher {
    client: reqwest::Client,
    retries: u32,
    base_delay: Duration,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl Fetcher {
    pub fn new(user_agent: &str, timeout: Duration, retries: u32) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .connect_timeout(timeout)
            .build()?;
        Ok(Self {
            client,
            retries,
            base_delay: Duration::from_millis(500),
            cache: Arc::default(),
        })
    }

    pub fn with_base_delay(mut self, d: Duration) -> Self {
        self.base_delay = d;
        self
    }

    /// GET com revalidação por ETag. Em 304 devolve o corpo em cache.
    pub async fn get_text(&self, url: &str) -> Result<Arc<String>> {
        let mut attempt = 0u32;
        loop {
            match self.try_get(url).await {
                Ok(b) => return Ok(b),
                Err(e) if attempt < self.retries && is_retryable(&e) => {
                    let delay = self.base_delay * 2u32.saturating_pow(attempt);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_get(&self, url: &str) -> Result<Arc<String>> {
        let cached = self.cache.lock().expect("cache lock").get(url).cloned();
        let mut req = self.client.get(url);
        if let Some(etag) = cached.as_ref().and_then(|c| c.etag.as_deref()) {
            req = req.header(reqwest::header::IF_NONE_MATCH, etag);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_MODIFIED {
            if let Some(c) = cached {
                return Ok(c.body);
            }
        }
        if !status.is_success() {
            return Err(Error::Status {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }
        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let body = Arc::new(resp.text().await?);
        self.cache.lock().expect("cache lock").insert(
            url.to_string(),
            CacheEntry {
                etag,
                body: body.clone(),
            },
        );
        Ok(body)
    }
}

fn is_retryable(e: &Error) -> bool {
    match e {
        Error::Http(_) => true,
        Error::Status { status, .. } => *status == 429 || *status >= 500,
        _ => false,
    }
}

/// Provider de dados reais do TSE.
#[derive(Debug, Clone)]
pub struct TseProvider {
    endpoints: Arc<Endpoints>,
    fetcher: Fetcher,
    /// Lista de municípios (da config ou baixada uma vez).
    municipios: Arc<Mutex<Vec<MunicipioRef>>>,
}

impl TseProvider {
    pub fn new(endpoints: Endpoints) -> Result<Self> {
        let fetcher = Fetcher::new(
            &endpoints.user_agent,
            Duration::from_secs(endpoints.timeout_seconds),
            endpoints.retries,
        )?;
        Ok(Self::with_fetcher(endpoints, fetcher))
    }

    pub fn with_fetcher(endpoints: Endpoints, fetcher: Fetcher) -> Self {
        let municipios = Arc::new(Mutex::new(endpoints.municipios.clone()));
        Self {
            endpoints: Arc::new(endpoints),
            fetcher,
            municipios,
        }
    }

    async fn municipios(&self) -> Result<Vec<MunicipioRef>> {
        let cached = self.municipios.lock().expect("lock").clone();
        if !cached.is_empty() {
            return Ok(cached);
        }
        let Some(url) = &self.endpoints.municipios_url else {
            return Ok(cached);
        };
        let list = parse_municipios(&self.fetcher.get_text(url).await?, &self.endpoints.uf)?;
        *self.municipios.lock().expect("lock") = list.clone();
        Ok(list)
    }

    /// Busca o estado (obrigatório) e os municípios. Falha do estado = erro;
    /// município com falha é ignorado se ao menos um outro funcionou (dado parcial é melhor que nenhum),
    /// mas se TODOS falharem com a lista não vazia, devolve erro.
    pub async fn fetch(&self) -> Result<Snapshot> {
        let body = self.fetcher.get_text(&self.endpoints.estado).await?;
        let estado = parse_abrangencia(&body, "RJ", "Rio de Janeiro", None)?;

        let lista = self.municipios().await?;
        let sem = Arc::new(Semaphore::new(self.endpoints.concurrency.max(1)));
        let mut set = JoinSet::new();
        for (i, m) in lista.iter().cloned().enumerate() {
            let url = self.endpoints.municipio_url(&m.codigo);
            let f = self.fetcher.clone();
            let sem = sem.clone();
            set.spawn(async move {
                let _permit = sem.acquire_owned().await.ok();
                let r: Result<Parsed> = async {
                    let b = f.get_text(&url).await?;
                    parse_abrangencia(&b, &m.codigo, &m.nome, m.ibge.as_deref())
                }
                .await;
                (i, r)
            });
        }
        let mut ok: Vec<(usize, Parsed)> = Vec::new();
        let mut last_err = None;
        while let Some(res) = set.join_next().await {
            match res {
                Ok((i, Ok(p))) => ok.push((i, p)),
                Ok((_, Err(e))) => last_err = Some(e),
                Err(e) => last_err = Some(Error::Parse(format!("tarefa abortada: {e}"))),
            }
        }
        if ok.is_empty() && !lista.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::Parse("nenhum município obtido".into())));
        }
        ok.sort_by_key(|(i, _)| *i);
        Ok(build_snapshot(
            estado,
            ok.into_iter().map(|(_, p)| p).collect(),
            Fonte::Tse,
        ))
    }
}
