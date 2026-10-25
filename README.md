# apuracao-rj

Painel desktop (Tauri v2) para acompanhar a apuração do **2º turno de 2026, cargo Presidente, no Rio de Janeiro** (estado e 92 municípios).

> **Aviso: projeto não oficial.** Não tem vínculo com o TSE nem com qualquer órgão público. Os números vêm de dados públicos do TSE e podem ter atraso ou divergência; para o resultado oficial, consulte sempre o TSE.

Leia em [English](README.en.md).

## Fonte dos dados
Dados públicos de divulgação de resultados do [TSE](https://www.tse.jus.br/); malha municipal do [IBGE](https://www.ibge.gov.br/). Veja [NOTICE.md](NOTICE.md) e `docs/DATA_SOURCES.md`.
O app é neutro: candidatos aparecem na ordem do TSE, com cores fixas por número. Quando a coleta falha, o dado é marcado como "desatualizado desde HH:MM".

## Como rodar em desenvolvimento
Requisitos: Node 24, Rust estável; no Linux, as dependências do webkit2gtk-4.1 para o app Tauri.

```bash
npm ci
npm run dev                      # navegador, com dados mock em memória
npm run tauri dev                # app desktop (use o modo mock nas configurações)
cargo test -p apuracao-core      # lógica de negócio
npx vitest run                   # testes do front
```

O modo mock usa `fixtures/mock/` e avança a apuração a cada ciclo.

## Como baixar
Instaladores para Linux, Windows e macOS (Intel e Apple Silicon) ficam em [Releases](../../releases).

## Como contribuir
Veja [CONTRIBUTING.md](CONTRIBUTING.md) e o [Código de Conduta](CODE_OF_CONDUCT.md). Divergências com o TSE: abra uma issue "Dado incorreto (TSE)".

## Licença
[GPL-3.0-or-later](LICENSE).
