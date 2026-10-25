# Contrato entre as equipes — apuracao-rj

App Tauri v2 (GPL-3.0-or-later) que mostra a apuração do 2º turno 2026, cargo PRESIDENTE, só no RJ (estado + 92 municípios). Dados públicos do TSE, app não oficial.

## Layout e donos dos arquivos (NÃO editar fora do seu domínio)
| Dono | Arquivos |
|---|---|
| dados | `docs/DATA_SOURCES.md`, `config/endpoints.json`, `fixtures/**` |
| backend | `Cargo.toml` (workspace), `crates/core/**`, `src-tauri/**` |
| frontend | `package.json`, `vite.config.ts`, `tsconfig*.json`, `index.html`, `src/**`, `public/**` |
| infra | `.github/**`, `LICENSE`, `README.md`, `README.en.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `NOTICE.md`, `deny.toml`, `.gitignore`, `.editorconfig` |

Todo arquivo-fonte leva `SPDX-License-Identifier: GPL-3.0-or-later`.
Ambiente: node 24, npm (sem pnpm), cargo 1.99. NÃO há webkit2gtk: `src-tauri` não compila aqui, por isso toda a lógica fica em `crates/core` (testável com `cargo test -p apuracao-core`) e `src-tauri` é só uma casca fina.

## Modelo normalizado (JSON, camelCase) — idêntico em Rust (serde) e TS
```ts
type Candidato = { numero: string; nome: string; partido: string; votos: number; percentual: number; /* % dos válidos, 0-100 */ eleito: boolean };
type Totais = { secoesTotais: number; secoesApuradas: number; percentualApurado: number; votosValidos: number; brancos: number; nulos: number; abstencoes: number; eleitorado: number };
type Abrangencia = { codigo: string; /* "RJ" ou código TSE do município */ nome: string; ibge?: string; totais: Totais; candidatos: Candidato[] /* ordenados por votos desc */ };
type Snapshot = { geradoEm: string; /* ISO, hora da coleta */ atualizadoTse: string; /* ISO, hora declarada pelo TSE */ fonte: "tse" | "mock"; estado: Abrangencia; municipios: Abrangencia[]; desatualizado: boolean };
type PontoHistorico = { em: string; percentualApurado: number; candidatos: { numero: string; votos: number }[] };
```

## Comandos Tauri (invoke) e eventos
- `get_snapshot(): Snapshot` — último snapshot em cache (ou busca se vazio).
- `refresh_now(): Snapshot` — força busca.
- `get_history(): PontoHistorico[]` — persistido em disco (app data dir).
- `get_settings()/set_settings({ pollSeconds: number, mock: boolean })`.
- Evento `apuracao:update` (payload `Snapshot`) a cada ciclo; evento `apuracao:error` (payload `string`) em falha (UI mostra "desatualizado desde HH:MM", nunca esconde dado velho como atual).
- Modo mock: `fixtures/mock/` com sequência de snapshots que "avança" a apuração a cada ciclo.

## Frontend
React + TypeScript + Vite. Abstrair o `invoke` em `src/api.ts`; se não estiver no Tauri (`window.__TAURI_INTERNALS__` ausente) usa mock em memória, para `npm run dev` e build funcionarem no navegador. Telas: painel do estado, mapa do RJ por município (SVG/d3-geo, GeoJSON IBGE em `public/geo/rj-municipios.json`), tabela de municípios (busca, ordenação, CSV), detalhe do município, gráfico de evolução, agrupamento Capital/Baixada/interior (opcional). Neutralidade: ordem dos candidatos = ordem do TSE no painel; cores fixas por número, sem juízo. Aviso legal e fonte sempre visíveis. pt-BR (en opcional).
