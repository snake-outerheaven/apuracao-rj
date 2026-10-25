// SPDX-License-Identifier: GPL-3.0-or-later
import { regiaoDe } from "../grouping";
import type { Abrangencia } from "../types";
import { BarraDisputa, Candidatos, TotaisApuracao } from "./StatePanel";

export function MunDetail({ m, onClose }: { m: Abrangencia | null; onClose: () => void }) {
  if (!m) return <section className="card" aria-labelledby="h-det"><h2 id="h-det">Detalhe do município</h2><p className="muted">Selecione um município no mapa ou na tabela.</p></section>;
  return (
    <section className="card" aria-labelledby="h-det" aria-live="polite">
      <div className="row">
        <h2 id="h-det">{m.nome}</h2>
        <button type="button" onClick={onClose} aria-label="Limpar seleção">×</button>
      </div>
      <p className="muted">Região: {regiaoDe(m.ibge)} · IBGE {m.ibge ?? "—"}</p>
      <BarraDisputa a={m} />
      <Candidatos a={m} />
      <TotaisApuracao a={m} />
    </section>
  );
}
