// SPDX-License-Identifier: GPL-3.0-or-later
import { geoMercator, geoPath } from "d3-geo";
import type { GeoPermissibleObjects } from "d3-geo";
import { useEffect, useMemo, useState } from "react";
import { corCandidato, fmtPct } from "../format";
import { lider } from "../grouping";
import type { Abrangencia } from "../types";

type Feat = { type: "Feature"; geometry: GeoPermissibleObjects; properties: { ibge: string; nome: string } };
type FC = { type: "FeatureCollection"; features: Feat[] };

const W = 800, H = 560;
const k6 = (s: string | undefined) => (s ?? "").slice(0, 6);

/** Carrega a malha IBGE; se falhar, devolve erro (o app segue funcionando sem o mapa). */
function useMalha(): { fc: FC | null; erro: string | null } {
  const [st, setSt] = useState<{ fc: FC | null; erro: string | null }>({ fc: null, erro: null });
  useEffect(() => {
    let vivo = true;
    fetch(`${import.meta.env.BASE_URL}geo/rj-municipios.json`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then((j: FC) => {
        if (!j || !Array.isArray(j.features)) throw new Error("formato inválido");
        if (vivo) setSt({ fc: j, erro: null });
      })
      .catch((e: unknown) => vivo && setSt({ fc: null, erro: String(e instanceof Error ? e.message : e) }));
    return () => { vivo = false; };
  }, []);
  return st;
}

export function RjMap({ municipios, selecionado, onSelect }: { municipios: Abrangencia[]; selecionado: string | null; onSelect: (ibge: string) => void }) {
  const { fc, erro } = useMalha();
  const paths = useMemo(() => {
    if (!fc) return [];
    const proj = geoMercator().fitSize([W, H], fc as unknown as GeoPermissibleObjects);
    const gp = geoPath(proj);
    return fc.features.map((f) => ({ f, d: gp(f.geometry) ?? "" }));
  }, [fc]);
  const porIbge = useMemo(() => new Map(municipios.map((m) => [k6(m.ibge), m])), [municipios]);

  if (erro) return <p className="aviso" role="status">Mapa indisponível ({erro}). A tabela continua funcionando. Veja public/geo/ no README.</p>;
  if (!fc) return <p className="muted">Carregando mapa…</p>;
  return (
    <figure className="mapa">
      <svg viewBox={`0 0 ${W} ${H}`} role="group" aria-label="Mapa do estado do Rio de Janeiro por município, colorido pelo candidato com mais votos">
        {paths.map(({ f, d }) => {
          const m = porIbge.get(k6(f.properties.ibge));
          const l = m ? lider(m) : null;
          const sel = selecionado !== null && k6(selecionado) === k6(f.properties.ibge);
          const op = m ? 0.35 + 0.65 * Math.min(1, m.totais.percentualApurado / 100) : 0.15;
          const rotulo = m ? `${f.properties.nome}: ${l ? `${l.nome} ${fmtPct(l.percentual)}` : "sem líder"}; ${fmtPct(m.totais.percentualApurado)} apurado` : f.properties.nome;
          return (
            <path
              key={f.properties.ibge}
              d={d}
              className={sel ? "mun sel" : "mun"}
              fill={l ? corCandidato(l.numero) : "var(--neutro)"}
              fillOpacity={op}
              tabIndex={0}
              role="button"
              aria-pressed={sel}
              aria-label={rotulo}
              onClick={() => onSelect(f.properties.ibge)}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onSelect(f.properties.ibge); } }}
            >
              <title>{rotulo}</title>
            </path>
          );
        })}
      </svg>
      <figcaption className="muted">Cor = candidato com mais votos no município; opacidade = % de seções apuradas. Malha: IBGE.</figcaption>
    </figure>
  );
}
