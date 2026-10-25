<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Fontes de dados — TSE (resultados.tse.jus.br)

Pesquisa feita em 2026-10-25 (dia do 2º turno) com `curl` direto nos arquivos públicos. Legenda:
**[CONFIRMADO]** = baixado com HTTP 200 e inspecionado; **[SUPOSTO]** = deduzido por analogia, não testado ou sem retorno.

## 1. Resumo e AVISOS importantes

- **[CONFIRMADO]** O padrão de 2022 do enunciado (`ele2022/545/dados-simplificados/rj/rj-c0001-e000545-r.json`) **NÃO funciona mais**: 404 em todas as variantes testadas (`dados-simplificados`, `dados`, `config`, sufixos `-r`, `-v`, `-u`). Também 404 para ele2024. Logo **não foi possível baixar amostras reais de 2022**; as fixtures reais vêm de 2026 (ver §6).
- **[CONFIRMADO]** O padrão vigente em 2026 usa diretório `dados/<uf>` e sufixo **`-u`**, descoberto via o índice `comum/config/ele-c.json` (campo `arq`, que dá os templates de diretório).
- **[CONFIRMADO]** Códigos 2026: ciclo `ele2026`; pleito 1º turno `3220`, 2º turno `3221` (25/10/2026); eleição **Presidente 1º turno = `6257`**, **Presidente 2º turno = `6258`** ("Eleição Ordinária Federal - 2026 2º Turno", cargo `1` Presidente).
- **[CONFIRMADO]** Em 2026-10-25 ~17h07 BRT, os arquivos do 2º turno existiam apenas como **modelo pré-apuração** (`dg/hg` = 13/10/2026 17:48, todos os votos 0, `snt` = seções totais). Ou seja: a **estrutura** do 2º turno é igual à do 1º, mas **valores reais de apuração do 2º turno não foram observados** — [SUPOSTO] que o mesmo arquivo seja sobrescrito durante a noite (comportamento idêntico ao do 1º turno, cujo arquivo final traz `hg` e `dt/ht` preenchidos).
- Baixar só: não há autenticação, nem cookies. Cabeçalhos observados: `cache-control: max-age=52`, `x-ratelimit-limit: 5000`, CDN Akamai.

## 2. URLs (todas sob `https://resultados.tse.jus.br/oficial`)

`cd_eleicao6` = zero-padding a 6 dígitos (6258 → `006258`). `cargo4` = `0001`. `cd_mun5` = código TSE do município com 5 dígitos.

| O quê | Template | Status |
|---|---|---|
| Índice de pleitos/eleições | `/comum/config/ele-c.json` | [CONFIRMADO] (26 KB) |
| Lista de municípios | `/ele2026/6258/config/mun-e006258-cm.json` | [CONFIRMADO] (534 KB, todas as UFs) |
| Resultado do estado | `/ele2026/6258/dados/rj/rj-c0001-e006258-u.json` | [CONFIRMADO] estrutura (1º turno com dados reais, 2º só modelo zerado) |
| Resultado de município | `/ele2026/6258/dados/rj/rj{cd_mun5}-c0001-e006258-u.json` | [CONFIRMADO] idem (~9 KB) |
| Abrangência (todos os 92 mun. numa chamada) | `/ele2026/6258/dados/rj/rj-e006258-ab.json` | [CONFIRMADO] (71 KB, 93 entradas) |
| Config de seções | `/ele2026/arquivo-urna/3221/config/rj/rj-p003221-cs.json` | [CONFIRMADO] existe (820 KB), não necessário |
| Fotos | `/ele2026/{cd_eleicao}/fotos/{uf}/...` | [SUPOSTO] (só aparece em `arq` do índice) |
| Sufixos `-r`, `-v`, `dados-simplificados/` | — | 404; não usar |

Machine-readable: `config/endpoints.json`.

**Estratégia de coleta sugerida**: a cada ciclo (≥ 60 s, o CDN cacheia ~52 s) baixar (a) `rj-c0001-e006258-u.json` e (b) `rj-e006258-ab.json`; comparar `ht`/`dt` por município no `ab` e só rebaixar `rj{mun}-...-u.json` dos municípios que mudaram (ou, mais simples, os 92 — ~830 KB por ciclo — respeitando o limite). Não há ETag condicional garantido; `last-modified`/`etag` estão presentes e podem ser usados com `If-None-Match` [SUPOSTO que funcione, não testado].

## 3. Estrutura do JSON de resultado (`-u.json`) — [CONFIRMADO]

Todos os valores numéricos vêm como **string**. Percentuais: versão `pvap` com vírgula decimal ("53,01") e versão `pvapn` com ponto e mais casas ("53,009637448", sufixo `n` = numérico). Use sempre a versão `...n`.

Raiz: `ele` (cd_eleição), `t` (turno), `f` (`o`=oficial), `tpabr` (`uf` no arquivo do estado, `mu` no do município), `cdabr` (`rj` ou cd_mun), `dg`/`hg` (data/hora **de geração do arquivo**, dd/mm/aaaa, hh:mm:ss, horário de Brasília), `dt`/`ht` (data/hora da **última totalização**; vazios antes de começar), `dv`, `tf` (totalização final `s`/`n`), `and` (andamento: `f` finalizada, `n` não iniciada, presumivelmente `p` parcial [SUPOSTO]), `esae`, `mnae`.

`carg[0]` (cargo Presidente, `cd="1"`): `fed[]` federações; `agr[]` agrupamentos → `par[]` partidos (`n` número, `sg` sigla, `nm`, `tvtn` total de votos do partido) → `cand[]`:
- `n` número, `nm` nome completo, `nmu` nome de urna, `sqcand` sequencial, `dt` nascimento, `seq` ordem no TSE
- `vap` **votos apurados** (string int), `pvap`/`pvapn` **% sobre votos válidos**
- `e` eleito (`s`/`n`), `st` status textual ("2º turno", "Não eleito", "Eleito"), `dvt` ("Válido")
- `vs[]` vice (`tp:"v"`, `nm`, `nmu`, `sgp`).

Observação: no 1º turno o JSON lista candidatos agrupados por partido/coligação, **não ordenados por votos**; ordenar no app (contrato pede votos desc no `Abrangencia`; a regra de neutralidade do painel usa ordem do TSE = `seq`).

Blocos fora de `carg` (raiz do arquivo, também em `ab`):
- `s` seções: `ts` totais, `st` totalizadas (apuradas), `pst`/`pstn` % totalizado, `snt` não totalizadas, `si`/`sni` instaladas, `sa`/`sna` ... (`a` = apuradas no sentido de arquivo de urna).
- `e` eleitorado: `te` total, `est` do eleitorado totalizado, `esnt` não totalizado, `c` **comparecimento**, `pc`/`pcn`, `a` **abstenções**, `pa`/`pan`.
- `v` votos (só no `-u`, não no `ab`): `tv` total comparecido, `vvc` votos válidos+... , `vv` **votos válidos**, `vnom` nominais, `vb` **brancos**, `pvbn`, `tvn` total nulos (`vn` nulos + `vnt` nulos técnicos), `van` anulados, `vansj`, `vsan`, `vscv`.

### Mapeamento → modelo do contrato
| Contrato | Origem |
|---|---|
| `Totais.secoesTotais` | `s.ts` |
| `Totais.secoesApuradas` | `s.st` |
| `Totais.percentualApurado` | `s.pstn` (número) ou `st/ts*100` |
| `Totais.votosValidos` | `v.vv` |
| `Totais.brancos` | `v.vb` |
| `Totais.nulos` | `v.tvn` (nulos totais) — [a confirmar com o TSE se `vn` sem `vnt` é preferível; no 1º turno RJ: tvn 288123 = vn 287736 + vnt 387] |
| `Totais.abstencoes` | `e.a` |
| `Totais.eleitorado` | `e.te` |
| `Candidato.votos` | `int(vap)` |
| `Candidato.percentual` | `float(pvapn)` |
| `Candidato.eleito` | `e == "s"` — CUIDADO: no 1º turno `e="s"` significa "vai ao 2º turno" (`st`="2º turno"); no 2º turno confiar em `st == "Eleito"` [SUPOSTO] |
| `Snapshot.atualizadoTse` | `dt`+`ht` (última totalização); se vazio, `dg`+`hg`. Fuso America/Sao_Paulo (UTC-3 em 2026, sem horário de verão) |
| `Abrangencia.codigo`/`ibge` | `cdabr` / `cdi` da lista de municípios |

## 4. Lista de municípios: código TSE → IBGE — [CONFIRMADO]

`/ele2026/6258/config/mun-e006258-cm.json`:
```json
{ "dg":"15/10/2026","hg":"09:38:00","abr":[ {"cd":"rj","ds":"RIO DE JANEIRO","mu":[
  {"cd":"58017","cdi":"3300100","nm":"ANGRA DOS REIS","c":"n","z":["0116","0147"]}, ... ]} ]}
```
`cd` = código TSE (5 dígitos, usado nas URLs), `cdi` = código IBGE de 7 dígitos, `nm` = nome em MAIÚSCULAS, `c`=`s` para capital (Rio de Janeiro: `60011` → `3304557`), `z` = zonas. RJ tem exatamente **92** municípios no arquivo. Fixture reduzida ao RJ: `fixtures/tse-2026-rj/mun-e006258-cm.rj-only.json`. Nenhum cruzamento externo é necessário; `public/geo/rj-municipios.json` (IBGE) casa por `cdi`.

## 5. Periodicidade — parcialmente confirmado

- [CONFIRMADO] `cache-control: max-age=52` → o CDN renova no máximo a cada ~1 min. Polling recomendado: **60 s** (não menos de 30 s).
- [CONFIRMADO] limite informado `x-ratelimit-limit: 5000` (janela `w=1`); 93 requisições/ciclo estão longe disso, mas o app deve espalhá-las (concorrência ≤ 8) e enviar `User-Agent` identificável.
- [SUPOSTO] Em 2022 o TSE atualizava os arquivos a cada poucos minutos desde as 17h (horário de Brasília; 16h no Acre/Amazonas-oeste não se aplica ao RJ). Divulgação do 2º turno: fechamento das urnas 17h BRT; dados do RJ começam ~17h-17h30.
- Sempre exibir `dt`/`ht` do arquivo como "atualizado pelo TSE" e marcar desatualizado se `hg` não avança.

## 6. Fixtures

### `fixtures/tse-2026-rj/` — amostras reais (públicas, baixadas em 2026-10-25)
(O pedido era `tse-2022-rj`; 2022 indisponível — ver §1. Pasta renomeada para refletir o que é real.)
- `turno1-final/`: 1º turno 2026 presidente RJ, 100% apurado, **dados reais completos**:
  `rj-c0001-e006257-u.json` (estado, 12 candidatos), `rj60011-…` (Rio), `rj58653-…` (Niterói), `rj58017-…` (Angra dos Reis), `rj-e006257-ab.json` (abrangência dos 92).
- `turno2-pre-apuracao/`: mesmos arquivos para `e006258` (2º turno) no estado pré-apuração (zerados; só 2 candidatos). Serve de teste de "apuração não iniciada" (`dt/ht` vazios, `snt = ts`).
- `mun-e006258-cm.rj-only.json`: lista de municípios filtrada para o RJ.
- Tamanho total ~250 KB.

### `fixtures/mock/` — sequência sintética
`node fixtures/mock/generate.mjs` (determinístico, sem dependências) gera `snapshots/01.json … 10.json` e `sequence.json` (array dos 10). Apuração do estado: 5,3 / 15,9 / 26,6 / 37,2 / 47,8 / 58,5 / 69,1 / 79,7 / 91,2 / 100 %. `fonte: "mock"`, candidatos "Candidato Teste A" (nº 91, TESTE-A) e "Candidata Teste B" (nº 92, TESTE-B), 92 municípios reais com `codigo` TSE e `ibge` reais, totais do estado = soma dos municípios, contagens monótonas, `eleito=true` só no último snapshot e só no estado. Os números são inventados.

## 7. Não confirmado / pendências

1. Valores reais do 2º turno 2026 durante a apuração (arquivo ainda era o modelo zerado).
2. Valor de `and` em apuração parcial; semântica exata de `e`/`st` no 2º turno.
3. Existência de `If-None-Match`/304; frequência real de atualização.
4. Padrão de 2022 (404) e fotos de candidatos.
5. Termos de uso/licença dos dados: o TSE publica como dados abertos (portal `dadosabertos.tse.jus.br`); a Res.-TSE 23.669/2021 arts. 231-237 trata da divulgação por terceiros (citado na página do TSE de "interessados na divulgação de resultados"); o app deve citar a fonte e dizer que não é oficial.
