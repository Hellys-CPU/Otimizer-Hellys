import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { Optimization } from "@/types";

export default function Security() {
  const [items, setItems] = useState<Optimization[]>([]);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{ id: string; ok: boolean; message: string } | null>(null);

  const reload = () =>
    api
      .listCatalog()
      .then((all) => setItems(all.filter((o) => o.categoria === "security")))
      .catch(() => setItems([]));

  useEffect(() => {
    reload();
  }, []);

  const handleApply = async (o: Optimization) => {
    const confirmed = window.confirm(
      `"${o.nome}" afeta um recurso de segurança do Windows.\n\n` +
        `Impacto de segurança: ${o.descricao}\n` +
        `Benefício de desempenho: ${o.beneficioEsperado}\n` +
        `Reinicialização necessária: ${o.requerReinicializacao ? "sim" : "não"}\n\n` +
        `Confirma a aplicação?`,
    );
    if (!confirmed) return;
    setBusyId(o.id);
    setFeedback(null);
    try {
      await api.applyOptimization(o.id);
      await reload();
      setFeedback({ id: o.id, ok: true, message: "Aplicado com sucesso." });
    } catch (err) {
      setFeedback({ id: o.id, ok: false, message: String(err) });
    } finally {
      setBusyId(null);
    }
  };

  const handleRestore = async (o: Optimization) => {
    setBusyId(o.id);
    setFeedback(null);
    try {
      await api.restoreOptimization(o.id);
      await reload();
      setFeedback({ id: o.id, ok: true, message: "Restaurado com sucesso." });
    } catch (err) {
      setFeedback({ id: o.id, ok: false, message: String(err) });
    } finally {
      setBusyId(null);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Segurança</h2>
      <div className="rounded-md border border-forge-warn/40 bg-forge-warn/10 p-3 text-sm text-forge-warn">
        Estes itens nunca fazem parte de nenhum perfil — não existe "aplicar todos".
        Cada um exige confirmação individual aqui, mostrando o impacto de segurança antes de
        qualquer alteração.
      </div>

      <div className="space-y-3">
        {items.map((o) => (
          <div key={o.id} className="rounded-lg border border-forge-danger/30 bg-forge-panel p-4">
            <div className="mb-2 flex items-center justify-between">
              <p className="font-medium">{o.nome}</p>
              <span className="rounded bg-forge-danger/20 px-2 py-0.5 text-xs text-forge-danger">
                {o.risco}
              </span>
            </div>
            <p className="mb-1 text-sm text-slate-400">{o.descricao}</p>
            <p className="mb-3 text-xs text-slate-500">
              Benefício esperado: {o.beneficioEsperado} · Reinicialização:{" "}
              {o.requerReinicializacao ? "sim" : "não"} · Fonte: {o.fonteTecnica}
            </p>
            {feedback?.id === o.id && (
              <p className={`mb-3 text-xs ${feedback.ok ? "text-forge-safe" : "text-forge-danger"}`}>
                {feedback.ok ? "✓ " : "✗ "}
                {feedback.message}
              </p>
            )}
            <div className="flex gap-2">
              <button
                disabled={busyId === o.id}
                onClick={() => handleApply(o)}
                className="rounded-md bg-forge-danger px-3 py-1.5 text-xs font-medium text-white hover:bg-forge-danger/80"
              >
                Aplicar (com confirmação)
              </button>
              <button
                disabled={busyId === o.id}
                onClick={() => handleRestore(o)}
                className="rounded-md border border-forge-border px-3 py-1.5 text-xs hover:bg-white/5"
              >
                Restaurar
              </button>
            </div>
          </div>
        ))}
        {items.length === 0 && (
          <p className="text-sm text-slate-500">Nenhum item de segurança carregado.</p>
        )}
      </div>
    </div>
  );
}
