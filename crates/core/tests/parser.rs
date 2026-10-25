// SPDX-License-Identifier: GPL-3.0-or-later
use apuracao_core::endpoints::Endpoints;
use apuracao_core::parser::{build_snapshot, parse_abrangencia, to_f64, to_u64, tse_datetime};
use apuracao_core::{Fonte, Snapshot};
use serde_json::json;

const ESTADO: &str = include_str!("data/estado_2022.json");
const MUNICIPIO: &str = include_str!("data/municipio_2022.json");

#[test]
fn numeros_em_string_e_virgula() {
    assert_eq!(to_f64(&json!("50,90")), Some(50.9));
    assert_eq!(to_f64(&json!("1.234,5")), Some(1234.5));
    assert_eq!(to_f64(&json!(12.5)), Some(12.5));
    assert_eq!(to_f64(&json!("")), None);
    assert_eq!(to_f64(&json!("abc")), None);
    assert_eq!(to_u64(&json!("1.234.567")), Some(1_234_567));
    assert_eq!(to_u64(&json!("12.500")), Some(12_500));
    assert_eq!(to_u64(&json!("60345999")), Some(60_345_999));
    assert_eq!(to_u64(&json!(42)), Some(42));
    assert_eq!(to_u64(&json!("-3")), None);
    assert_eq!(to_u64(&json!(null)), None);
}

#[test]
fn data_tse_para_iso() {
    assert_eq!(
        tse_datetime(Some("30/10/2022"), Some("19:15:31")).as_deref(),
        Some("2022-10-30T19:15:31-03:00")
    );
    assert_eq!(
        tse_datetime(Some("30/10/2022"), None).as_deref(),
        Some("2022-10-30T00:00:00-03:00")
    );
    assert_eq!(tse_datetime(Some("lixo"), None), None);
}

#[test]
fn parseia_estado() {
    let p = parse_abrangencia(ESTADO, "RJ", "Rio de Janeiro", None).unwrap();
    let a = p.abrangencia;
    assert_eq!(a.codigo, "RJ");
    assert_eq!(a.nome, "Rio de Janeiro");
    assert_eq!(a.totais.secoes_totais, 30_000);
    assert_eq!(a.totais.secoes_apuradas, 29_700);
    assert!((a.totais.percentual_apurado - 99.0).abs() < 1e-9);
    assert_eq!(a.totais.votos_validos, 10_095_000);
    assert_eq!(a.totais.brancos, 100_000);
    assert_eq!(a.totais.nulos, 305_000);
    assert_eq!(a.totais.abstencoes, 2_000_000);
    assert_eq!(a.candidatos.len(), 2);
    assert_eq!(a.candidatos[0].numero, "22");
    assert_eq!(a.candidatos[0].votos, 5_200_000);
    assert!((a.candidatos[0].percentual - 51.51).abs() < 1e-9);
    assert!(!a.candidatos[0].eleito);
    assert_eq!(
        p.atualizado_tse.as_deref(),
        Some("2022-10-30T19:15:31-03:00")
    );
}

#[test]
fn parseia_municipio_com_campos_ausentes() {
    let p = parse_abrangencia(MUNICIPIO, "60011", "Angra dos Reis", Some("3300100")).unwrap();
    let a = p.abrangencia;
    assert_eq!(a.ibge.as_deref(), Some("3300100"));
    assert_eq!(a.totais.eleitorado, 0); // ausente
    assert!((a.totais.percentual_apurado - 100.0).abs() < 1e-9); // calculado de st/ts
    assert_eq!(a.totais.nulos, 2000); // sem tvn: vn + vnt(ausente)
                                      // ordenado por votos desc e pvap ausente calculado
    assert_eq!(a.candidatos[0].numero, "13");
    assert!(a.candidatos[0].eleito);
    assert!((a.candidatos[1].percentual - 40.0).abs() < 1e-9);
}

#[test]
fn json_vazio_ou_invalido_e_erro() {
    assert!(parse_abrangencia("{}", "X", "X", None).is_err());
    assert!(parse_abrangencia("[]", "X", "X", None).is_err());
    assert!(parse_abrangencia("não é json", "X", "X", None).is_err());
}

#[test]
fn snapshot_roundtrip_camel_case() {
    let e = parse_abrangencia(ESTADO, "RJ", "Rio de Janeiro", None).unwrap();
    let m = parse_abrangencia(MUNICIPIO, "60011", "Angra dos Reis", Some("3300100")).unwrap();
    let s = build_snapshot(e, vec![m], Fonte::Tse);
    assert_eq!(s.atualizado_tse, "2022-10-30T19:15:31-03:00");
    assert!(!s.desatualizado);
    let v = serde_json::to_value(&s).unwrap();
    assert!(v.get("geradoEm").is_some());
    assert!(v["estado"]["totais"].get("secoesApuradas").is_some());
    assert_eq!(v["fonte"], "tse");
    assert!(v["municipios"][0].get("ibge").is_some());
    let back: Snapshot = serde_json::from_value(v).unwrap();
    assert_eq!(back, s);
}

fn fx(p: &str) -> String {
    std::fs::read_to_string(format!("{}/../../fixtures/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn endpoints_real_expande_templates() {
    let t = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/endpoints.json"
    ))
    .unwrap();
    let e = Endpoints::from_json(&t).unwrap();
    assert_eq!(
        e.estado,
        "https://resultados.tse.jus.br/oficial/ele2026/6258/dados/rj/rj-c0001-e006258-u.json"
    );
    assert_eq!(
        e.municipio_url("60011"),
        "https://resultados.tse.jus.br/oficial/ele2026/6258/dados/rj/rj60011-c0001-e006258-u.json"
    );
    assert!(e
        .municipios_url
        .unwrap()
        .ends_with("/ele2026/6258/config/mun-e006258-cm.json"));
}

#[test]
fn lista_de_municipios_2026() {
    let l = apuracao_core::endpoints::parse_municipios(
        &fx("tse-2026-rj/mun-e006258-cm.rj-only.json"),
        "rj",
    )
    .unwrap();
    assert_eq!(l.len(), 92);
    assert_eq!(l[0].nome, "Angra dos Reis");
    assert_eq!(l[0].ibge.as_deref(), Some("3300100"));
}

#[test]
fn parseia_turno1_final_2026() {
    let p = parse_abrangencia(
        &fx("tse-2026-rj/turno1-final/rj-c0001-e006257-u.json"),
        "RJ",
        "Rio de Janeiro",
        None,
    )
    .unwrap();
    let a = p.abrangencia;
    assert_eq!(a.candidatos.len(), 2.max(a.candidatos.len()));
    assert!(a.candidatos.windows(2).all(|w| w[0].votos >= w[1].votos));
    assert!(a
        .candidatos
        .iter()
        .all(|c| !c.partido.is_empty() && !c.nome.is_empty()));
    assert!(!a.candidatos.iter().any(|c| c.eleito)); // "2º turno" não é eleito
    assert_eq!(a.totais.nulos, 288_123);
    assert!(a.totais.eleitorado > 0 && a.totais.abstencoes > 0 && a.totais.votos_validos > 0);
    assert!((a.totais.percentual_apurado - 100.0).abs() < 1e-6);
    let soma: f64 = a.candidatos.iter().map(|c| c.percentual).sum();
    assert!(soma > 99.0 && soma < 100.01, "soma {soma}");
    assert!(p.atualizado_tse.unwrap().starts_with("2026-10-"));
}

#[test]
fn parseia_municipio_turno1_2026() {
    let p = parse_abrangencia(
        &fx("tse-2026-rj/turno1-final/rj60011-c0001-e006257-u.json"),
        "60011",
        "Rio de Janeiro",
        Some("3304557"),
    )
    .unwrap();
    assert_eq!(p.abrangencia.totais.secoes_totais, 12_635);
    assert_eq!(p.abrangencia.totais.eleitorado, 4_952_612);
    assert_eq!(p.abrangencia.totais.nulos, 113_584);
    assert_eq!(
        p.abrangencia.candidatos[0].votos,
        p.abrangencia
            .candidatos
            .iter()
            .map(|c| c.votos)
            .max()
            .unwrap()
    );
}

#[test]
fn pre_apuracao_turno2_zerada() {
    let p = parse_abrangencia(
        &fx("tse-2026-rj/turno2-pre-apuracao/rj-c0001-e006258-u.json"),
        "RJ",
        "Rio de Janeiro",
        None,
    )
    .unwrap();
    let a = p.abrangencia;
    assert_eq!(a.totais.percentual_apurado, 0.0);
    assert!(a
        .candidatos
        .iter()
        .all(|c| c.votos == 0 && c.percentual == 0.0 && !c.eleito));
    assert_eq!(
        p.atualizado_tse.as_deref(),
        Some("2026-10-13T17:48:47-03:00")
    ); // dt vazio => dg/hg
}

#[test]
fn mock_fixtures_carregam() {
    let mut m = apuracao_core::mock::MockProvider::from_dir(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/mock"),
    )
    .unwrap();
    let a = m.avancar();
    let b = m.avancar();
    assert_eq!(a.fonte, Fonte::Mock);
    assert!(b.estado.totais.percentual_apurado >= a.estado.totais.percentual_apurado);
}
