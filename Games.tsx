import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import { useAppStore } from "@/store/useAppStore";
import type { DetectedGame } from "@/types";

export default function Games() {
  const { profiles, loadProfiles } = useAppStore();
  const [games, setGames] = useState<DetectedGame[]>([]);
  const [form, setForm] = useState({ nome: "", executableName: "", launcher: "custom" });

  const reload = () => api.listDetectedGames().then(setGames).catch(() => setGames([]));

  useEffect(() => {
    loadProfiles();
    reload();
  }, [loadProfiles]);

  const handleRegister = async () => {
    if (!form.nome || !form.executableName) return;
    await api.registerGame({
      nome: form.nome,
      executableName: form.executableName,
      launcher: form.launcher,
    });
    setForm({ nome: "", executableName: "", launcher: "custom" });
    reload();
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Jogos</h2>
      <div className="rounded-md border border-forge-border/60 bg-white/5 p-3 text-sm text-slate-400">
        Associe um perfil a um jogo. Ao detectar o processo do executável em execução, o
        SystemForge aplica o perfil automaticamente e restaura o estado anterior quando o jogo
        fecha (checagem a cada 5s).
      </div>

      <div className="flex gap-2 rounded-lg border border-forge-border bg-forge-panel p-4">
        <input
          className="flex-1 rounded-md border border-forge-border bg-forge-bg px-2 py-1.5 text-sm"
          placeholder="Nome do jogo"
          value={form.nome}
          onChange={(e) => setForm({ ...form, nome: e.target.value })}
        />
        <input
          className="flex-1 rounded-md border border-forge-border bg-forge-bg px-2 py-1.5 text-sm"
          placeholder="executavel.exe"
          value={form.executableName}
          onChange={(e) => setForm({ ...form, executableName: e.target.value })}
        />
        <select
          className="rounded-md border border-forge-border bg-forge-bg px-2 py-1.5 text-sm"
          value={form.launcher}
          onChange={(e) => setForm({ ...form, launcher: e.target.value })}
        >
          {["steam", "epic", "xbox", "battlenet", "riot", "custom"].map((l) => (
            <option key={l} value={l}>
              {l}
            </option>
          ))}
        </select>
        <button
          onClick={handleRegister}
          className="rounded-md bg-forge-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-forge-accent/80"
        >
          Adicionar
        </button>
      </div>

      <div className="space-y-2">
        {games.map((g) => (
          <div
            key={g.id}
            className="flex items-center justify-between rounded-lg border border-forge-border bg-forge-panel p-3"
          >
            <div>
              <p className="font-medium">{g.nome}</p>
              <p className="text-xs text-slate-500">
                {g.executableName} · {g.launcher ?? "—"}
                {g.lastPlayedAt && ` · última vez: ${new Date(g.lastPlayedAt).toLocaleString("pt-BR")}`}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <select
                className="rounded-md border border-forge-border bg-forge-bg px-2 py-1 text-xs"
                value={g.profileId ?? ""}
                onChange={(e) => api.setGameProfile(g.id, e.target.value || null).then(reload)}
              >
                <option value="">Sem perfil (não detecta)</option>
                {profiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.nome}
                  </option>
                ))}
              </select>
              <button
                onClick={() => api.deleteGame(g.id).then(reload)}
                className="rounded-md border border-forge-border px-2 py-1 text-xs hover:bg-white/5"
              >
                Remover
              </button>
            </div>
          </div>
        ))}
        {games.length === 0 && (
          <p className="text-sm text-slate-500">Nenhum jogo cadastrado ainda.</p>
        )}
      </div>
    </div>
  );
}
