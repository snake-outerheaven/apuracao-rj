// SPDX-License-Identifier: GPL-3.0-or-later
//! Modelo normalizado (contrato com o frontend). JSON em camelCase.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidato {
    pub numero: String,
    pub nome: String,
    pub partido: String,
    pub votos: u64,
    /// Percentual dos votos válidos, 0-100.
    pub percentual: f64,
    pub eleito: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Totais {
    pub secoes_totais: u64,
    pub secoes_apuradas: u64,
    pub percentual_apurado: f64,
    pub votos_validos: u64,
    pub brancos: u64,
    pub nulos: u64,
    pub abstencoes: u64,
    pub eleitorado: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Abrangencia {
    /// "RJ" ou código TSE do município.
    pub codigo: String,
    pub nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibge: Option<String>,
    pub totais: Totais,
    /// Ordenados por votos (desc).
    pub candidatos: Vec<Candidato>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fonte {
    Tse,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// ISO-8601, hora da coleta.
    pub gerado_em: String,
    /// ISO-8601, hora declarada pelo TSE.
    pub atualizado_tse: String,
    pub fonte: Fonte,
    pub estado: Abrangencia,
    pub municipios: Vec<Abrangencia>,
    #[serde(default)]
    pub desatualizado: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VotoHistorico {
    pub numero: String,
    pub votos: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PontoHistorico {
    pub em: String,
    pub percentual_apurado: f64,
    pub candidatos: Vec<VotoHistorico>,
}

impl PontoHistorico {
    /// Ponto de histórico do total do estado.
    pub fn from_snapshot(s: &Snapshot) -> Self {
        Self {
            em: s.atualizado_tse.clone(),
            percentual_apurado: s.estado.totais.percentual_apurado,
            candidatos: s
                .estado
                .candidatos
                .iter()
                .map(|c| VotoHistorico {
                    numero: c.numero.clone(),
                    votos: c.votos,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub poll_seconds: u64,
    pub mock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_seconds: 60,
            mock: false,
        }
    }
}

impl Settings {
    pub const MIN_POLL_SECONDS: u64 = 30;
    pub const MAX_POLL_SECONDS: u64 = 3600;

    /// Limita o intervalo a valores sensatos (evita martelar o TSE).
    pub fn sanitized(mut self) -> Self {
        self.poll_seconds = self
            .poll_seconds
            .clamp(Self::MIN_POLL_SECONDS, Self::MAX_POLL_SECONDS);
        self
    }
}
