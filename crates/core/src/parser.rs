// SPDX-License-Identifier: GPL-3.0-or-later
//! Parser tolerante do JSON de resultados do TSE (formato 2026, confirmado em docs/DATA_SOURCES.md:
//! `carg[0].agr[].par[].cand[]` com `n/nmu/vap/pvapn/st`; totais em `s` (ts/st/pstn), `e` (te/a) e
//! `v` (vv/vb/tvn); `dt/ht` = última totalização, `dg/hg` = geração do arquivo). Também aceita o formato 2022 (`cand` na raiz).
//!
//! Regras: campos ausentes viram 0/""/false; números podem vir como número JSON ou string,
//! com vírgula decimal ("50,90") e/ou ponto de milhar ("1.234.567").
//! Também aceita nomes longos do próprio modelo (camelCase) como sinônimos.

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::model::{Abrangencia, Candidato, Fonte, Snapshot, Totais};

/// Resultado do parse de um arquivo: a abrangência e o horário declarado pelo TSE (ISO), se houver.
#[derive(Debug, Clone)]
pub struct Parsed {
    pub abrangencia: Abrangencia,
    pub atualizado_tse: Option<String>,
}

// ---------- números ----------

/// Converte número JSON ou string (vírgula decimal, milhar com ponto) em f64.
pub fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => str_to_f64(s),
        _ => None,
    }
}

fn str_to_f64(s: &str) -> Option<f64> {
    let t: String = s
        .trim()
        .trim_end_matches('%')
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if t.is_empty() {
        return None;
    }
    let norm = if t.contains(',') {
        t.replace('.', "").replace(',', ".")
    } else if t.matches('.').count() > 1 {
        t.replace('.', "") // "1.234.567"
    } else {
        t
    };
    norm.parse::<f64>().ok().filter(|f| f.is_finite())
}

/// Inteiro não-negativo. "1.234" (um ponto, 3 dígitos depois) é tratado como milhar.
pub fn to_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().filter(|f| *f >= 0.0).map(|f| f.round() as u64)),
        Value::String(s) => {
            let t = s.trim();
            let thousands = t.matches('.').count() == 1
                && !t.contains(',')
                && t.rsplit('.')
                    .next()
                    .is_some_and(|d| d.len() == 3 && d.chars().all(|c| c.is_ascii_digit()))
                && t.split('.')
                    .next()
                    .is_some_and(|d| !d.is_empty() && d.len() <= 3);
            let f = if thousands {
                str_to_f64(&t.replace('.', ""))
            } else {
                str_to_f64(t)
            }?;
            (f >= 0.0).then(|| f.round() as u64)
        }
        _ => None,
    }
}

fn to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

// ---------- busca de campos ----------

struct Scopes<'a>(Vec<&'a Value>);

impl<'a> Scopes<'a> {
    fn find(&self, keys: &[&str]) -> Option<&'a Value> {
        self.0
            .iter()
            .find_map(|s| keys.iter().find_map(|k| s.get(*k).filter(|v| !v.is_null())))
    }
    fn u64(&self, keys: &[&str]) -> Option<u64> {
        self.find(keys).and_then(to_u64)
    }
    fn f64(&self, keys: &[&str]) -> Option<f64> {
        self.find(keys).and_then(to_f64)
    }
    fn string(&self, keys: &[&str]) -> Option<String> {
        self.find(keys)
            .and_then(to_string)
            .filter(|s| !s.is_empty())
    }
}

fn first_of<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v.get(key)? {
        Value::Array(a) => a.first(),
        o @ Value::Object(_) => Some(o),
        _ => None,
    }
}

/// Localiza candidatos em `cand`, `abr[].cand`, `carg[].agr[].par[].cand`; devolve (candidato, sigla do partido pai).
fn find_candidates<'a>(
    root: &'a Value,
    sg: Option<&'a str>,
    out: &mut Vec<(&'a Value, Option<&'a str>)>,
) {
    match root {
        Value::Object(m) => {
            let sg = m.get("sg").and_then(Value::as_str).or(sg);
            if let Some(Value::Array(c)) = m.get("cand").or_else(|| m.get("candidatos")) {
                out.extend(c.iter().map(|c| (c, sg)));
                return;
            }
            for k in ["abr", "carg", "agr", "par"] {
                if let Some(child) = m.get(k) {
                    find_candidates(child, sg, out);
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| find_candidates(x, sg, out)),
        _ => {}
    }
}

// ---------- datas ----------

fn brt() -> FixedOffset {
    FixedOffset::east_opt(-3 * 3600).expect("offset válido")
}

/// "30/10/2022" + "19:15:31" -> "2022-10-30T19:15:31-03:00". Aceita ISO pronto.
pub fn tse_datetime(data: Option<&str>, hora: Option<&str>) -> Option<String> {
    let data = data?.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(data) {
        return Some(dt.to_rfc3339());
    }
    let d = NaiveDate::parse_from_str(data, "%d/%m/%Y")
        .or_else(|_| NaiveDate::parse_from_str(data, "%Y-%m-%d"))
        .ok()?;
    let h = hora
        .map(str::trim)
        .and_then(|h| {
            NaiveTime::parse_from_str(h, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(h, "%H:%M"))
                .ok()
        })
        .unwrap_or_default();
    d.and_time(h)
        .and_local_timezone(brt())
        .single()
        .map(|dt| dt.to_rfc3339())
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

// ---------- parse ----------

/// Faz o parse de um arquivo de resultado (estado ou município).
/// `codigo`/`nome` são usados se o JSON não os trouxer (e `codigo` prevalece, por ser estável).
pub fn parse_abrangencia(
    text: &str,
    codigo: &str,
    nome: &str,
    ibge: Option<&str>,
) -> Result<Parsed> {
    let root: Value = serde_json::from_str(text)?;
    if !root.is_object() {
        return Err(Error::Parse("raiz do JSON não é um objeto".into()));
    }
    let node = first_of(&root, "abr").unwrap_or(&root);
    let carg = first_of(node, "carg").or_else(|| first_of(&root, "carg"));

    let mut scopes = vec![node];
    for k in ["s", "e", "v"] {
        if let Some(o) = node.get(k).filter(|v| v.is_object()) {
            scopes.push(o);
        }
    }
    scopes.push(&root);
    for k in ["s", "e", "v"] {
        if let Some(o) = root.get(k).filter(|v| v.is_object()) {
            scopes.push(o);
        }
    }
    if let Some(c) = carg {
        scopes.push(c);
    }
    let sc = Scopes(scopes);

    let mut raw = Vec::new();
    find_candidates(&root, None, &mut raw);

    let mut candidatos: Vec<Candidato> = raw
        .iter()
        .map(|(c, sg)| {
            let cs = Scopes(vec![*c]);
            let st = cs
                .string(&["st", "status"])
                .unwrap_or_default()
                .to_lowercase();
            let flag = cs
                .string(&["e", "eleito"])
                .unwrap_or_default()
                .to_lowercase();
            Candidato {
                numero: cs.string(&["n", "numero", "nr"]).unwrap_or_default(),
                nome: cs.string(&["nmu", "nome", "nm"]).unwrap_or_default(),
                partido: cs
                    .string(&["sgp", "partido"])
                    .or_else(|| sg.map(str::to_string))
                    .unwrap_or_default(),
                votos: cs.u64(&["vap", "votos", "v"]).unwrap_or(0),
                percentual: cs
                    .f64(&["pvapn", "pvap", "percentual", "pv"])
                    .unwrap_or(f64::NAN),
                // `e == "s"` no 1º turno significa "vai ao 2º turno": havendo `st`, ele manda.
                eleito: if st.is_empty() {
                    matches!(flag.as_str(), "s" | "true" | "sim")
                } else {
                    st.starts_with("eleito")
                },
            }
        })
        .filter(|c| !c.numero.is_empty() || !c.nome.is_empty())
        .collect();

    let soma: u64 = candidatos.iter().map(|c| c.votos).sum();
    let secoes_totais = sc.u64(&["ts", "secoesTotais"]).unwrap_or(0);
    // `st` = seções totalizadas no nível da abrangência (no candidato é "status").
    let secoes_apuradas = sc.u64(&["st", "secoesApuradas", "sa"]).unwrap_or(0);
    let percentual_apurado = sc
        .f64(&["pstn", "pst", "percentualApurado"])
        .unwrap_or_else(|| {
            if secoes_totais > 0 {
                secoes_apuradas as f64 * 100.0 / secoes_totais as f64
            } else {
                0.0
            }
        });
    let votos_validos = sc.u64(&["vv", "votosValidos"]).unwrap_or(soma);
    let nulos = sc
        .u64(&["tvn", "nulos"])
        .unwrap_or_else(|| sc.u64(&["vn"]).unwrap_or(0) + sc.u64(&["vnt"]).unwrap_or(0));
    // `e` pode ser o objeto de eleitorado (já tratado como escopo) ou o próprio número.
    let eleitorado = sc
        .u64(&["te", "eleitorado"])
        .or_else(|| root.get("e").and_then(to_u64))
        .or_else(|| node.get("e").and_then(to_u64))
        .unwrap_or(0);

    let base = votos_validos.max(1) as f64;
    for c in &mut candidatos {
        if !c.percentual.is_finite() {
            c.percentual = if votos_validos > 0 {
                c.votos as f64 * 100.0 / base
            } else {
                0.0
            };
        }
    }
    candidatos.sort_by_key(|c| std::cmp::Reverse(c.votos));

    let totais = Totais {
        secoes_totais,
        secoes_apuradas,
        percentual_apurado,
        votos_validos,
        brancos: sc.u64(&["vb", "brancos"]).unwrap_or(0),
        nulos,
        abstencoes: sc.u64(&["a", "abstencoes"]).unwrap_or(0),
        eleitorado,
    };

    if candidatos.is_empty() && totais == Totais::default() {
        return Err(Error::Parse("nenhum candidato ou total encontrado".into()));
    }

    // Última totalização (dt/ht) se houver; senão geração do arquivo (dg/hg).
    let atualizado_tse = tse_datetime(
        root.get("dt").and_then(to_string).as_deref(),
        root.get("ht").and_then(to_string).as_deref(),
    )
    .or_else(|| {
        tse_datetime(
            sc.string(&["dg", "atualizadoTse"]).as_deref(),
            sc.string(&["hg"]).as_deref(),
        )
    });

    Ok(Parsed {
        abrangencia: Abrangencia {
            codigo: codigo.to_string(),
            nome: if nome.is_empty() {
                sc.string(&["ds", "nome"]).unwrap_or_default()
            } else {
                nome.to_string()
            },
            ibge: ibge.map(str::to_string),
            totais,
            candidatos,
        },
        atualizado_tse,
    })
}

/// Monta o `Snapshot` a partir do estado e dos municípios já parseados.
pub fn build_snapshot(estado: Parsed, municipios: Vec<Parsed>, fonte: Fonte) -> Snapshot {
    let atualizado = estado
        .atualizado_tse
        .clone()
        .or_else(|| {
            municipios
                .iter()
                .filter_map(|m| m.atualizado_tse.clone())
                .max()
        })
        .unwrap_or_else(now_iso);
    Snapshot {
        gerado_em: now_iso(),
        atualizado_tse: atualizado,
        fonte,
        estado: estado.abrangencia,
        municipios: municipios.into_iter().map(|m| m.abrangencia).collect(),
        desatualizado: false,
    }
}
