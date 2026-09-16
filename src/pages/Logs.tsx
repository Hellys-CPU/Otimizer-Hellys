import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { ExecutionLogEntry } from "@/types";

export default function Logs() {
  const [logs, setLogs] = useState<ExecutionLogEntry[]>([]);

  useEffect(() => {
    api
      .listLogs()
      .then(setLogs)
      .catch(() => setLogs([]));
  }, []);

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Logs</h2>
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-forge-border text-left text-slate-400">
            <th className="py-2">Comando</th>
            <th>Usuário</th>
            <th>Data</th>
            <th>Resultado</th>
            <th>Código</th>
            <th>Reinício</th>
          </tr>
        </thead>
        <tbody>
          {logs.map((l) => (
            <tr key={l.id} className="border-b border-forge-border/50">
              <td className="py-2 font-mono text-xs">{l.command}</td>
              <td>{l.user}</td>
              <td>{new Date(l.timestamp).toLocaleString("pt-BR")}</td>
              <td className={l.result === "success" ? "text-forge-safe" : "text-forge-danger"}>
                {l.result}
              </td>
              <td>{l.returnCode ?? "—"}</td>
              <td>{l.requiresReboot ? "Sim" : "Não"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
