# Como contribuir

1. Abra uma issue (bug, sugestão ou dado incorreto do TSE) antes de mudanças grandes.
2. Faça fork, crie um branch e abra um PR usando o template.
3. Antes do PR rode:
   - `cargo fmt --all` e `cargo clippy -p apuracao-core --all-targets -- -D warnings`
   - `cargo test -p apuracao-core`
   - `npm ci && npx tsc --noEmit && npm run build && npx vitest run`
4. Todo arquivo-fonte deve ter `SPDX-License-Identifier: GPL-3.0-or-later`.
5. Lógica de negócio fica em `crates/core`; `src-tauri` é uma casca fina.
6. Neutralidade: não altere a ordem dos candidatos (ordem do TSE) nem use cores/textos que sugiram juízo.

Ao contribuir, você concorda em licenciar sua contribuição sob GPL-3.0-or-later.
