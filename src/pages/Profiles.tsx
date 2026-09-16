import { useEffect } from "react";
import { useAppStore } from "@/store/useAppStore";

export default function Profiles() {
  const { profiles, loadProfiles, applyProfile, restoreProfile, loading, error } = useAppStore();

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Perfis</h2>
      {error && (
        <div className="rounded-md border border-forge-danger/40 bg-forge-danger/10 p-3 text-sm text-forge-danger">
          {error}
        </div>
      )}
      <div className="grid grid-cols-2 gap-4">
        {profiles.map((p) => (
          <div key={p.id} className="rounded-lg border border-forge-border bg-forge-panel p-4">
            <div className="flex items-start justify-between">
              <div>
                <p className="font-medium">{p.nome}</p>
                <p className="text-sm text-slate-400">{p.descricao}</p>
              </div>
              <span
                className={`rounded px-2 py-0.5 text-xs ${
                  p.nivelAgressividade === 3
                    ? "bg-forge-danger/20 text-forge-danger"
                    : p.nivelAgressividade === 2
                      ? "bg-forge-warn/20 text-forge-warn"
                      : "bg-forge-safe/20 text-forge-safe"
                }`}
              >
                Nível {p.nivelAgressividade}
              </span>
            </div>
            <p className="mt-2 text-xs text-slate-500">
              {p.otimizacoesIds.length} otimizações · {p.requerAdministrador ? "requer admin" : "sem admin"}
            </p>
            <div className="mt-3 flex gap-2">
              <button
                disabled={loading}
                onClick={() => applyProfile(p.id)}
                className="rounded-md bg-forge-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-forge-accent/80"
              >
                Aplicar
              </button>
              <button
                disabled={loading}
                onClick={() => restoreProfile(p.id)}
                className="rounded-md border border-forge-border px-3 py-1.5 text-xs hover:bg-white/5"
              >
                Restaurar
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
