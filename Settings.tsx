import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";

export default function Settings() {
  const [telemetryOptIn, setTelemetryOptIn] = useState<boolean | null>(null);

  useEffect(() => {
    api
      .getTelemetryOptIn()
      .then(setTelemetryOptIn)
      .catch(() => setTelemetryOptIn(false));
  }, []);

  const toggle = async () => {
    if (telemetryOptIn === null) return;
    const next = !telemetryOptIn;
    await api.setTelemetryOptIn(next);
    setTelemetryOptIn(next);
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Configurações</h2>

      <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
        <h3 className="mb-1 text-sm font-medium text-slate-300">Privacidade e telemetria</h3>
        <p className="mb-3 text-sm text-slate-400">
          As métricas de antes/depois (tela "Comparação") já são salvas <strong>somente neste
          computador</strong>, sempre — isso não depende deste botão. Esta chave é reservada para
          uma futura opção de envio agregado e anônimo para melhorar recomendações entre usuários;
          hoje, mesmo ativada, nenhum código de rede a lê e nada é enviado.
        </p>
        <div className="flex items-center gap-3">
          <button
            onClick={toggle}
            disabled={telemetryOptIn === null}
            className={`rounded-md px-3 py-1.5 text-sm font-medium ${
              telemetryOptIn
                ? "bg-forge-safe/20 text-forge-safe"
                : "bg-white/5 text-slate-300"
            }`}
          >
            {telemetryOptIn === null ? "Carregando..." : telemetryOptIn ? "Ativada" : "Desativada"}
          </button>
          <span className="text-xs text-slate-500">
            Nenhum dado de Registro, serviços ou identidade é ou será coletado — apenas métricas
            agregadas de desempenho (CPU/RAM/FPS quando disponível).
          </span>
        </div>
      </div>
    </div>
  );
}
