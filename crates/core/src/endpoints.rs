// SPDX-License-Identifier: GPL-3.0-or-later
//! Leitura de `config/endpoints.json` (agente "dados") e da lista de municípios do TSE
//! (`mun-e006258-cm.json`: `abr[].cd == "rj"` -> `mu[]` com `cd` TSE, `cdi` IBGE, `nm`).
//!
//! As URLs vêm de `urls.resultado_uf.template` e `urls.resultado_municipio.template`, com os
//! placeholders `{base} {ambiente} {ciclo} {cd_eleicao} {cd_eleicao6} {uf} {cargo4}`;
//! `{cd_mun5}` vira `{codigo}` (preenchido a cada município).

use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct MunicipioRef {
    /// Código TSE (5 dígitos).
    pub codigo: String,
    pub nome: String,
    pub ibge: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Endpoints {
    pub estado: String,
    /// URL com `{codigo}`.
    pub municipio: String,
    /// URL da lista de municípios (usada se `municipios` estiver vazio).
    pub municipios_url: Option<String>,
    pub municipios: Vec<MunicipioRef>,
    pub uf: String,
    pub user_agent: String,
    pub timeout_seconds: u64,
    pub retries: u32,
    pub concurrency: usize,
}

impl Endpoints {
    pub fn new(estado: impl Into<String>, municipio: impl Into<String>) -> Self {
        Self {
            estado: estado.into(),
            municipio: municipio.into(),
            municipios_url: None,
            municipios: Vec::new(),
            uf: "rj".into(),
            user_agent: format!(
                "apuracao-rj/{} (app não oficial)",
                env!("CARGO_PKG_VERSION")
            ),
            timeout_seconds: 15,
            retries: 3,
            concurrency: 8,
        }
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(text)?;
        let s = |p: &[&str]| -> Option<String> {
            p.iter()
                .try_fold(&v, |acc, k| acc.get(*k))
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        let req = |p: &[&str]| {
            s(p).ok_or_else(|| Error::Config(format!("endpoints.json: falta {}", p.join("."))))
        };
        let uf = req(&["alvo", "uf"])?;
        let eleicao = req(&["eleicao", "presidente_turno2"])?;
        let vars = [
            ("{base}", req(&["base"])?),
            ("{ambiente}", req(&["ambiente"])?),
            ("{ciclo}", req(&["ciclo"])?),
            ("{cd_eleicao6}", format!("{:0>6}", eleicao)),
            ("{cd_eleicao}", eleicao.clone()),
            ("{uf}", uf.clone()),
            ("{cargo4}", req(&["alvo", "cargo"])?),
        ];
        let expand = |t: String| vars.iter().fold(t, |acc, (k, val)| acc.replace(k, val));
        let estado = expand(req(&["urls", "resultado_uf", "template"])?);
        let municipio = expand(req(&["urls", "resultado_municipio", "template"])?)
            .replace("{cd_mun5}", "{codigo}");
        if !municipio.contains("{codigo}") || estado.contains('{') {
            return Err(Error::Config(
                "endpoints.json: templates com placeholders desconhecidos".into(),
            ));
        }
        let mut e = Self::new(estado, municipio);
        e.municipios_url = s(&["urls", "config_municipios", "template"]).map(expand);
        e.uf = uf;
        Ok(e)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }

    pub fn municipio_url(&self, codigo: &str) -> String {
        self.municipio.replace("{codigo}", codigo)
    }
}

/// Extrai os municípios da UF de um `mun-e...-cm.json`.
pub fn parse_municipios(text: &str, uf: &str) -> Result<Vec<MunicipioRef>> {
    let v: Value = serde_json::from_str(text)?;
    let abr = v
        .get("abr")
        .and_then(Value::as_array)
        .and_then(|a| {
            a.iter().find(|x| {
                x.get("cd")
                    .and_then(Value::as_str)
                    .is_some_and(|c| c.eq_ignore_ascii_case(uf))
            })
        })
        .ok_or_else(|| Error::Parse(format!("UF {uf} ausente na lista de municípios")))?;
    let out: Vec<_> = abr
        .get("mu")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|m| {
            Some(MunicipioRef {
                codigo: m.get("cd")?.as_str()?.to_string(),
                nome: title_case(m.get("nm").and_then(Value::as_str).unwrap_or_default()),
                ibge: m.get("cdi").and_then(Value::as_str).map(str::to_string),
            })
        })
        .collect();
    if out.is_empty() {
        return Err(Error::Parse("lista de municípios vazia".into()));
    }
    Ok(out)
}

/// "ANGRA DOS REIS" -> "Angra dos Reis".
fn title_case(s: &str) -> String {
    const MIN: [&str; 6] = ["de", "da", "do", "das", "dos", "e"];
    s.split_whitespace()
        .enumerate()
        .map(|(i, w)| {
            let l = w.to_lowercase();
            if i > 0 && MIN.contains(&l.as_str()) {
                return l;
            }
            let mut c = l.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
