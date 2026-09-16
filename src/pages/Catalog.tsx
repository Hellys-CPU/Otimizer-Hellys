import { useEffect, useState } from "react";
import { useAppStore } from "@/store/useAppStore";
import { api } from "@/lib/tauri";

export default function Catalog() {
  const { catalog, loadCatalog, loading } = useAppStore();
  const [busyId, setBusyId] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{ id: string; ok: boolean; message: string } | null>(null);

  useEffect(() => {
    loadCatalog();
  }, [loadCatalog]);

  const handleApply = async (id: string) => {
    setBusyId(id);
    setFeedback(null);
    try {
      await api.applyOptimization(id);
      await loadCatalog();
      setFeedback({ id, ok: true, message: "Aplicado com sucesso." });
    } catch (err) {
      setFeedback({ id, ok: false, message: String(err) });
    } finally {
      setBusyId(null);
    }
  };

  const handleRestore = async (id: string) => {
    setBusyId(id);
    setFeedback(null);
    try {
      await api.restoreOptimization(id);
      await loadCatalog();
      setFeedback({ id, ok: true, message: "Restaurado com sucesso." });
    } catch (err) {
      setFeedback({ id, ok: false, message: String(err) });
    } finally {
      setBusyId(null);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Catálogo de otimizações</h2>
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-forge-border text-left text-slate-400">
            <th className="py-2">Nome</th>
            <th>Categoria</th>
            <th>Risco</th>
            <th>Reinício</th>
            <th>Atual</th>
            <th>Recomendado</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {catalog.map((o) => (
            <tr key={o.id} className="border-b border-forge-border/50">
              <td className="py-2">
                <p>{o.nome}</p>
                <p className="text-xs text-slate-500">{o.descricao}</p>
                {feedback?.id === o.id && (
                  <p className={`mt-1 text-xs ${feedback.ok ? "text-forge-safe" : "text-forge-danger"}`}>
                    {feedback.ok ? "✓ " : "✗ "}
                    {feedback.message}
                  </p>
                )}
              </td>
              <td>{o.categoria}</td>
              <td>
                <span
                  className={
                    o.risco === "experimental"
                      ? "text-forge-danger"
                      : o.risco === "advanced"
                        ? "text-forge-warn"
                        : "text-forge-safe"
                  }
                >
                  {o.risco}
                </span>
              </td>
              <td>{o.requerReinicializacao ? "Sim" : "Não"}</td>
              <td className="text-slate-400">{o.valorAtual ?? "—"}</td>
              <td className="text-slate-400">{o.valorRecomendado}</td>
              <td className="flex gap-2 py-2">
                <button
                  disabled={loading || busyId === o.id}
                  onClick={() => handleApply(o.id)}
                  className="rounded-md bg-forge-accent px-2 py-1 text-xs text-white hover:bg-forge-accent/80"
                >
                  Aplicar
                </button>
                <button
                  disabled={loading || busyId === o.id}
                  onClick={() => handleRestore(o.id)}
                  className="rounded-md border border-forge-border px-2 py-1 text-xs hover:bg-white/5"
                >
                  Reverter
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
