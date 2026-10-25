// SPDX-License-Identifier: GPL-3.0-or-later
mod common;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use apuracao_core::history::History;
use apuracao_core::mock::MockProvider;
use apuracao_core::parser::{build_snapshot, parse_abrangencia};
use apuracao_core::service::{Service, ServiceConfig};
use apuracao_core::{Fonte, Settings, Snapshot};

const ESTADO: &str = include_str!("data/estado_2022.json");

fn snap(votos: &str) -> Snapshot {
    let txt = ESTADO.replace("5.200.000", votos);
    build_snapshot(
        parse_abrangencia(&txt, "RJ", "Rio de Janeiro", None).unwrap(),
        vec![],
        Fonte::Tse,
    )
}

#[test]
fn historico_persiste_e_ignora_repetido() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sub/h.json");
    let mut h = History::load(&path);
    assert!(h.record(&snap("5.200.000")).unwrap());
    assert!(!h.record(&snap("5.200.000")).unwrap());
    assert!(h.record(&snap("5.300.000")).unwrap());
    let h2 = History::load(&path);
    assert_eq!(h2.pontos().len(), 2);
    assert_eq!(h2.pontos()[1].candidatos[0].votos, 5_300_000);
}

#[test]
fn historico_corrompido_vira_vazio() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("h.json");
    std::fs::write(&path, "{{{").unwrap();
    assert!(History::load(&path).pontos().is_empty());
}

#[test]
fn mock_avanca_e_para_no_ultimo() {
    let mut m = MockProvider::from_frames(vec![snap("5.200.000"), snap("5.300.000")]).unwrap();
    assert_eq!(m.avancar().estado.candidatos[0].votos, 5_200_000);
    assert_eq!(m.avancar().fonte, Fonte::Mock);
    assert_eq!(m.avancar().estado.candidatos[0].votos, 5_300_000); // permanece no último
    assert!(MockProvider::from_frames(vec![]).is_err());
}

fn cfg(dir: &std::path::Path) -> ServiceConfig {
    ServiceConfig {
        data_dir: dir.join("data"),
        endpoints_path: dir.join("nao-existe.json"),
        municipios_path: dir.join("nao-existe-cm.json"),
        mock_dir: dir.join("mock"),
    }
}

#[tokio::test]
async fn service_mock_e_historico() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("mock")).unwrap();
    for (i, v) in ["5.000.000", "5.100.000"].iter().enumerate() {
        std::fs::write(
            dir.path().join(format!("mock/{i:02}.json")),
            serde_json::to_vec(&snap(v)).unwrap(),
        )
        .unwrap();
    }
    let svc = Service::new(cfg(dir.path()));
    svc.set_settings(Settings {
        poll_seconds: 1,
        mock: true,
    })
    .unwrap();
    assert_eq!(svc.settings().poll_seconds, Settings::MIN_POLL_SECONDS); // saneado
    let a = svc.get_snapshot().await.unwrap();
    assert_eq!(a.fonte, Fonte::Mock);
    let b = svc.get_snapshot().await.unwrap(); // cache, não avança
    assert_eq!(a.estado.candidatos[0].votos, b.estado.candidatos[0].votos);
    let c = svc.refresh().await.unwrap();
    assert!(c.estado.candidatos[0].votos > a.estado.candidatos[0].votos);
    assert_eq!(svc.history().len(), 2);
    // configurações persistem
    let svc2 = Service::new(cfg(dir.path()));
    assert!(svc2.settings().mock);
}

#[tokio::test]
async fn falha_do_tse_marca_desatualizado() {
    let dir = tempfile::tempdir().unwrap();
    let fail = Arc::new(AtomicBool::new(false));
    let f2 = fail.clone();
    let url = common::serve(move |_req| {
        if f2.load(Ordering::SeqCst) {
            common::response(503, &[], "indisponível")
        } else {
            common::response(200, &[], ESTADO)
        }
    })
    .await;
    let ep = dir.path().join("endpoints.json");
    std::fs::write(&ep, format!(r#"{{"base":"{url}","ambiente":"oficial","ciclo":"c","eleicao":{{"presidente_turno2":"6258"}},"alvo":{{"uf":"rj","cargo":"0001"}},"urls":{{"resultado_uf":{{"template":"{{base}}/rj.json"}},"resultado_municipio":{{"template":"{{base}}/m/{{cd_mun5}}.json"}}}}}}"#)).unwrap();
    let mut c = cfg(dir.path());
    c.endpoints_path = ep;
    let svc = Service::new(c);
    let ok = svc.get_snapshot().await.unwrap();
    assert!(!ok.desatualizado);
    assert_eq!(ok.fonte, Fonte::Tse);
    fail.store(true, Ordering::SeqCst);
    assert!(svc.refresh().await.is_err());
    let cached = svc.get_snapshot().await.unwrap();
    assert!(cached.desatualizado);
    assert_eq!(cached.estado.candidatos, ok.estado.candidatos); // dado velho preservado, mas sinalizado
                                                                // sem endpoints e sem cache: erro
    let svc2 = Service::new(cfg(dir.path()));
    assert!(svc2.refresh().await.is_err());
}
