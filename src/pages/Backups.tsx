import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { ExecutionSession } from "@/types";

export default function Backups() {
  const [sessions, setSessions] = useState<ExecutionSession[]>([]);

  useEffect(() => {
    api
      .listSessions()
      .then(setSessions)
      .catch(() => setSessions([]));
  }, []);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Backup e restauração</h2>
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-forge-border text-left text-slate-400">
            <th className="py-2">Sessão</th>
            <th>Perfil</th>
            <th>Início</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {sessions.map((s) => (
            <tr key={s.id} className="border-b border-forge-border/50">
              <td className="py-2 font-mono text-xs">{s.id}</td>
              <td>{s.profileId}</td>
              <td>{new Date(s.startedAt).toLocaleString("pt-BR")}</td>
              <td>{s.status}</td>
              <td>
                <button
                  onClick={() => api.restoreSession(s.id)}
                  className="rounded-md border border-forge-border px-2 py-1 text-xs hover:bg-white/5"
                >
                  Restaurar sessão
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
