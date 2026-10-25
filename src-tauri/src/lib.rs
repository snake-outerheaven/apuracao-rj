// SPDX-License-Identifier: GPL-3.0-or-later
//! Casca fina do Tauri: expõe `apuracao-core` ao frontend. Nenhuma regra de negócio aqui.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use apuracao_core::service::{Service, ServiceConfig};
use apuracao_core::{PontoHistorico, Settings, Snapshot};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Notify;

const EVENT_UPDATE: &str = "apuracao:update";
const EVENT_ERROR: &str = "apuracao:error";

struct AppState {
    svc: Arc<Service>,
    /// Acorda o agendador (refresh manual ou mudança de configuração).
    wake: Arc<Notify>,
}

/// Um ciclo: busca e emite `apuracao:update` ou `apuracao:error`.
async fn cycle(app: &AppHandle, svc: &Service) -> Result<Snapshot, String> {
    match svc.refresh().await {
        Ok(snap) => {
            let _ = app.emit(EVENT_UPDATE, &snap);
            Ok(snap)
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = app.emit(EVENT_ERROR, &msg);
            Err(msg)
        }
    }
}

#[tauri::command]
async fn get_snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    state.svc.get_snapshot().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_now(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    cycle(&app, &state.svc).await
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<PontoHistorico> {
    state.svc.history()
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.svc.settings()
}

#[tauri::command]
fn set_settings(settings: Settings, state: State<'_, AppState>) -> Result<Settings, String> {
    let saved = state
        .svc
        .set_settings(settings)
        .map_err(|e| e.to_string())?;
    state.wake.notify_one();
    Ok(saved)
}

/// Primeiro caminho existente entre o diretório de recursos empacotados e a árvore do projeto (dev).
fn resolve(app: &tauri::App, rel: &str) -> PathBuf {
    let bundled = app.path().resource_dir().map(|d| d.join(rel)).ok();
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(rel);
    match bundled {
        Some(p) if p.exists() => p,
        _ => dev,
    }
}

pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_notification::init());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .setup(|app| {
            let cfg = ServiceConfig {
                data_dir: app.path().app_data_dir()?,
                endpoints_path: resolve(app, "config/endpoints.json"),
                municipios_path: resolve(app, "config/mun-e006258-cm.json"),
                mock_dir: resolve(app, "fixtures/mock"),
            };
            let svc = Arc::new(Service::new(cfg));
            let wake = Arc::new(Notify::new());
            app.manage(AppState {
                svc: svc.clone(),
                wake: wake.clone(),
            });

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = cycle(&handle, &svc).await;
                    let wait = Duration::from_secs(svc.settings().poll_seconds);
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {}
                        _ = wake.notified() => {}
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            refresh_now,
            get_history,
            get_settings,
            set_settings
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o apuracao-rj");
}
