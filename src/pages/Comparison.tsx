import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { TelemetryComparison } from "@/types";

function fmt(value: number | null, unit: string) {
  return value === null ? "não disponível" : `${value.toFixed(1)}${unit}`;
}

function delta(before: number | null, after: number | null) {
  if (before === null || after === null) return null;
  return after - before;
}

export default function Comparison() {
  const [rows, setRows] = useState<TelemetryComparison[]>([]);

  useEffect(() => {
    api
      .listTelemetryComparisons()
      .then(setRows)
      .catch(() => setRows([]));
  }, []);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Comparação antes e depois</h2>
      <p className="text-sm text-slate-400">
        Métricas capturadas automaticamente ao iniciar e ao fechar um jogo com perfil associado.
        CPU e RAM são sempre medições reais do sistema. FPS/1% low usam o{" "}
        <a
          href="https://github.com/GameTechDev/PresentMon"
          target="_blank"
          rel="noreferrer"
          className="underline"
        >
          PresentMon
        </a>{" "}
        se ele estiver instalado e no PATH — aparecem como "não disponível" caso contrário. Uso de
        GPU, temperatura e latência de entrada ainda não têm captura implementada.
      </p>
      {rows.length === 0 && (
        <p className="text-sm text-slate-500">
          Nenhuma sessão de jogo registrada ainda. Associe um perfil a um jogo na tela de Jogos.
        </p>
      )}
      <div className="space-y-3">
        {rows.map((r) => {
          const cpuDelta = delta(r.beforeCpuUsagePercent, r.afterCpuUsagePercent);
          const ramDelta = delta(r.beforeRamUsageMb, r.afterRamUsageMb);
          return (
            <div key={r.gameSessionId} className="rounded-lg border border-forge-border bg-forge-panel p-4">
              <div className="mb-3 flex items-center justify-between">
                <p className="font-medium">{r.gameName}</p>
                <p className="text-xs text-slate-500">
                  {new Date(r.startedAt).toLocaleString("pt-BR")}
                  {r.endedAt ? ` — ${new Date(r.endedAt).toLocaleString("pt-BR")}` : " (em andamento)"}
                </p>
              </div>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-xs text-slate-500">FPS médio</p>
                  <p>
                    {fmt(r.beforeFpsAvg, "")} → {fmt(r.afterFpsAvg, "")}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-slate-500">1% low</p>
                  <p>
                    {fmt(r.beforeFpsLow1pct, "")} → {fmt(r.afterFpsLow1pct, "")}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-slate-500">CPU do sistema</p>
                  <p>
                    {fmt(r.beforeCpuUsagePercent, "%")} → {fmt(r.afterCpuUsagePercent, "%")}
                    {cpuDelta !== null && (
                      <span className={cpuDelta <= 0 ? "text-forge-safe" : "text-forge-warn"}>
                        {" "}
                        ({cpuDelta >= 0 ? "+" : ""}
                        {cpuDelta.toFixed(1)}pp)
                      </span>
                    )}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-slate-500">RAM usada</p>
                  <p>
                    {fmt(r.beforeRamUsageMb, "MB")} → {fmt(r.afterRamUsageMb, "MB")}
                    {ramDelta !== null && (
                      <span className={ramDelta <= 0 ? "text-forge-safe" : "text-forge-warn"}>
                        {" "}
                        ({ramDelta >= 0 ? "+" : ""}
                        {ramDelta.toFixed(0)}MB)
                      </span>
                    )}
                  </p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
