import { useEffect, useState } from "react";
import { useAppStore } from "@/store/useAppStore";
import { api } from "@/lib/tauri";

interface SystemStatus {
  cpuUsage: number;
  ramUsage: number;
  windowsBuild: string;
  isAdmin: boolean;
}

export default function Dashboard() {
  const { profiles, loadProfiles, applyProfile, restoreProfile, loading, error } = useAppStore();
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [cleaning, setCleaning] = useState(false);
  const [cleanupResult, setCleanupResult] = useState<string | null>(null);

  useEffect(() => {
    loadProfiles();
    const poll = async () => {
      try {
        setStatus(await api.systemStatus());
      } catch {
        // backend ainda não implementou o comando nesta fase do scaffold
      }
    };
    poll();
    const id = setInterval(poll, 3000);
    return () => clearInterval(id);
  }, [loadProfiles]);

  const activeProfile = profiles.find((p) => p.ativo);

  const handleCleanTemp = async () => {
    if (!window.confirm("Apagar arquivos temporários do usuário (%TEMP%)? Isso não pode ser desfeito.")) {
      return;
    }
    setCleaning(true);
    setCleanupResult(null);
    try {
      const report = await api.cleanTempFiles();
      const mb = (report.bytesFreed / 1024 / 1024).toFixed(1);
      setCleanupResult(
        `${report.filesDeleted} arquivo(s) removido(s), ${mb} MB liberados` +
          (report.errors > 0 ? ` (${report.errors} arquivo(s) em uso, ignorado(s))` : ""),
      );
    } catch (err) {
      setCleanupResult(`Erro: ${String(err)}`);
    } finally {
      setCleaning(false);
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Dashboard</h2>

      {error && (
        <div className="rounded-md border border-forge-danger/40 bg-forge-danger/10 p-3 text-sm text-forge-danger">
          {error}
        </div>
      )}

      <div className="grid grid-cols-4 gap-4">
        <StatCard label="CPU" value={status ? `${status.cpuUsage.toFixed(0)}%` : "—"} />
        <StatCard label="RAM" value={status ? `${status.ramUsage.toFixed(0)}%` : "—"} />
        <StatCard label="Windows" value={status?.windowsBuild ?? "—"} />
        <StatCard label="Admin" value={status ? (status.isAdmin ? "Sim" : "Não") : "—"} />
      </div>

      <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
        <h3 className="mb-2 text-sm font-medium text-slate-300">Perfil ativo</h3>
        {activeProfile ? (
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">{activeProfile.nome}</p>
              <p className="text-sm text-slate-400">{activeProfile.descricao}</p>
            </div>
            <button
              disabled={loading}
              onClick={() => restoreProfile(activeProfile.id)}
              className="rounded-md border border-forge-border px-3 py-1.5 text-sm hover:bg-white/5"
            >
              Restaurar padrão
            </button>
          </div>
        ) : (
          <p className="text-sm text-slate-400">Nenhum perfil aplicado. Selecione um em Perfis.</p>
        )}
      </div>

      <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-medium text-slate-300">Limpeza de arquivos temporários</h3>
            <p className="text-xs text-slate-500">
              Apaga apenas %TEMP% do usuário atual. Não mexe em arquivos de sistema, não é reversível.
            </p>
          </div>
          <button
            disabled={cleaning}
            onClick={handleCleanTemp}
            className="rounded-md border border-forge-border px-3 py-1.5 text-sm hover:bg-white/5"
          >
            {cleaning ? "Limpando..." : "Limpar agora"}
          </button>
        </div>
        {cleanupResult && <p className="mt-2 text-xs text-slate-400">{cleanupResult}</p>}
      </div>

      <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
        <h3 className="mb-3 text-sm font-medium text-slate-300">Perfis disponíveis</h3>
        <div className="grid grid-cols-3 gap-3">
          {profiles.map((p) => (
            <div key={p.id} className="rounded-md border border-forge-border p-3">
              <p className="font-medium">{p.nome}</p>
              <p className="mb-2 text-xs text-slate-400">Nível {p.nivelAgressividade}</p>
              <button
                disabled={loading}
                onClick={() => applyProfile(p.id)}
                className="w-full rounded-md bg-forge-accent px-2 py-1 text-xs font-medium text-white hover:bg-forge-accent/80"
              >
                Aplicar
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
      <p className="text-xs text-slate-400">{label}</p>
      <p className="text-lg font-semibold">{value}</p>
    </div>
  );
}
