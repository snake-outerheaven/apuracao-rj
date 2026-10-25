// SPDX-License-Identifier: GPL-3.0-or-later
mod common;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use apuracao_core::endpoints::{Endpoints, MunicipioRef};
use apuracao_core::fetcher::{Fetcher, TseProvider};

const ESTADO: &str = include_str!("data/estado_2022.json");
const MUNICIPIO: &str = include_str!("data/municipio_2022.json");

fn fetcher(retries: u32) -> Fetcher {
    Fetcher::new("teste", Duration::from_secs(5), retries)
        .unwrap()
        .with_base_delay(Duration::from_millis(5))
}

#[tokio::test]
async fn etag_304_usa_cache() {
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let base = common::serve(move |req| {
        h.fetch_add(1, Ordering::SeqCst);
        if req.to_lowercase().contains("if-none-match: \"v1\"") {
            common::response(304, &[], "")
        } else {
            common::response(200, &[("ETag", "\"v1\"")], "corpo")
        }
    })
    .await;
    let f = fetcher(0);
    assert_eq!(*f.get_text(&format!("{base}/a")).await.unwrap(), "corpo");
    assert_eq!(*f.get_text(&format!("{base}/a")).await.unwrap(), "corpo"); // via 304
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn retry_com_backoff_em_5xx() {
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let base = common::serve(move |_| {
        if h.fetch_add(1, Ordering::SeqCst) < 2 {
            common::response(503, &[], "")
        } else {
            common::response(200, &[], "ok")
        }
    })
    .await;
    assert_eq!(*fetcher(3).get_text(&base).await.unwrap(), "ok");
    assert_eq!(hits.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn nao_repete_4xx_e_desiste_apos_retries() {
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let base = common::serve(move |_| {
        h.fetch_add(1, Ordering::SeqCst);
        common::response(404, &[], "")
    })
    .await;
    assert!(fetcher(3).get_text(&base).await.is_err());
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    let h2 = Arc::new(AtomicUsize::new(0));
    let h3 = h2.clone();
    let base = common::serve(move |_| {
        h3.fetch_add(1, Ordering::SeqCst);
        common::response(500, &[], "")
    })
    .await;
    assert!(fetcher(2).get_text(&base).await.is_err());
    assert_eq!(h2.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn timeout() {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        let (_s, _) = l.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
    });
    let f = Fetcher::new("t", Duration::from_millis(200), 0).unwrap();
    assert!(f.get_text(&format!("http://{addr}/")).await.is_err());
}

#[tokio::test]
async fn provider_tse_monta_snapshot() {
    let base = common::serve(|req| {
        if req.starts_with("GET /rj.json") {
            common::response(200, &[], ESTADO)
        } else if req.starts_with("GET /m/60011.json") {
            common::response(200, &[], MUNICIPIO)
        } else {
            common::response(404, &[], "")
        }
    })
    .await;
    let mut ep = Endpoints::new(
        format!("{base}/rj.json"),
        format!("{base}/m/{{codigo}}.json"),
    );
    ep.retries = 0;
    ep.municipios = vec![
        MunicipioRef {
            codigo: "60011".into(),
            nome: "Angra dos Reis".into(),
            ibge: Some("3300100".into()),
        },
        MunicipioRef {
            codigo: "99999".into(),
            nome: "Sumiu".into(),
            ibge: None,
        },
    ];
    let snap = TseProvider::new(ep).unwrap().fetch().await.unwrap();
    assert_eq!(snap.estado.codigo, "RJ");
    assert_eq!(snap.municipios.len(), 1); // o 404 é ignorado
    assert_eq!(snap.municipios[0].nome, "Angra dos Reis");
}

#[tokio::test]
async fn provider_falha_se_estado_falha() {
    let base = common::serve(|_| common::response(500, &[], "")).await;
    let mut ep = Endpoints::new(format!("{base}/rj.json"), "x/{codigo}");
    ep.retries = 0;
    assert!(TseProvider::new(ep).unwrap().fetch().await.is_err());
}

#[tokio::test]
async fn provider_baixa_lista_de_municipios_quando_nao_ha_arquivo() {
    let lista = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/tse-2026-rj/mun-e006258-cm.rj-only.json"
    ))
    .unwrap();
    let base = common::serve(move |req| {
        if req.starts_with("GET /rj.json") {
            common::response(200, &[], ESTADO)
        } else if req.starts_with("GET /cm.json") {
            common::response(200, &[], &lista)
        } else {
            common::response(200, &[], MUNICIPIO)
        }
    })
    .await;
    let mut ep = Endpoints::new(
        format!("{base}/rj.json"),
        format!("{base}/m/{{codigo}}.json"),
    );
    ep.municipios_url = Some(format!("{base}/cm.json"));
    ep.retries = 0;
    let snap = TseProvider::new(ep).unwrap().fetch().await.unwrap();
    assert_eq!(snap.municipios.len(), 92);
    assert!(snap.municipios.iter().all(|m| m.ibge.is_some()));
}
