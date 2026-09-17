import { useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type {
  StartupAppDiag,
  ServiceDiag,
  DiskDiag,
  PhysicalDiskDiag,
  GpuDiag,
  DriverDiag,
} from "@/types";

function Section({
  title,
  error,
  children,
}: {
  title: string;
  error?: string | null;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-lg border border-forge-border bg-forge-panel p-4">
      <h3 className="mb-3 text-sm font-medium text-slate-300">{title}</h3>
      {error ? <p className="text-xs text-forge-danger">Erro: {error}</p> : children}
    </div>
  );
}

export default function Diagnostics() {
  const [startupApps, setStartupApps] = useState<StartupAppDiag[] | null>(null);
  const [startupAppsError, setStartupAppsError] = useState<string | null>(null);
  const [services, setServices] = useState<ServiceDiag[] | null>(null);
  const [servicesError, setServicesError] = useState<string | null>(null);
  const [disks, setDisks] = useState<DiskDiag[] | null>(null);
  const [disksError, setDisksError] = useState<string | null>(null);
  const [physicalDisks, setPhysicalDisks] = useState<PhysicalDiskDiag[] | null>(null);
  const [physicalDisksError, setPhysicalDisksError] = useState<string | null>(null);
  const [gpus, setGpus] = useState<GpuDiag[] | null>(null);
  const [gpusError, setGpusError] = useState<string | null>(null);
  const [drivers, setDrivers] = useState<DriverDiag[] | null>(null);
  const [driversError, setDriversError] = useState<string | null>(null);
  const [powerScheme, setPowerScheme] = useState<string | null>(null);
  const [powerSchemeError, setPowerSchemeError] = useState<string | null>(null);
  const [exportMsg, setExportMsg] = useState<string | null>(null);
  const [restoreMsg, setRestoreMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [dpcCapturing, setDpcCapturing] = useState(false);
  const [dpcResult, setDpcResult] = useState<{ etlPath: string; csvPath: string | null } | string | null>(
    null,
  );

  useEffect(() => {
    api.listStartupApps().then(setStartupApps).catch((e) => setStartupAppsError(String(e)));
    api.listServicesDiagnostic().then(setServices).catch((e) => setServicesError(String(e)));
    api.listDisks().then(setDisks).catch((e) => setDisksError(String(e)));
    api.listPhysicalDisks().then(setPhysicalDisks).catch((e) => setPhysicalDisksError(String(e)));
    api.listGpus().then(setGpus).catch((e) => setGpusError(String(e)));
    api.listDrivers().then(setDrivers).catch((e) => setDriversError(String(e)));
    api.getCurrentPowerScheme().then(setPowerScheme).catch((e) => setPowerSchemeError(String(e)));
  }, []);

  const handleExport = async () => {
    setBusy(true);
    setExportMsg(null);
    try {
      const path = await api.exportDiagnostics();
      setExportMsg(`Salvo em: ${path}`);
    } catch (err) {
      setExportMsg(`Erro: ${String(err)}`);
    } finally {
      setBusy(false);
    }
  };

  const handleRestorePoint = async () => {
    if (
      !window.confirm(
        "Criar um ponto de restauração do Windows agora? O Windows limita a frequência (geralmente 1 por 24h) — se já criou um recentemente, isso pode falhar.",
      )
    ) {
      return;
    }
    setBusy(true);
    setRestoreMsg(null);
    try {
      await api.createRestorePoint("SystemForge Optimizer - ponto manual");
      setRestoreMsg("Ponto de restauração criado com sucesso.");
    } catch (err) {
      setRestoreMsg(`Erro: ${String(err)}`);
    } finally {
      setBusy(false);
    }
  };

  const handleCaptureDpcIsr = async () => {
    setDpcCapturing(true);
    setDpcResult(null);
    try {
      const result = await api.captureDpcIsr(15);
      setDpcResult(result);
    } catch (err) {
      setDpcResult(String(err));
    } finally {
      setDpcCapturing(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Diagnóstico</h2>
        <div className="flex gap-2">
          <button
            disabled={busy}
            onClick={handleRestorePoint}
            className="rounded-md border border-forge-border px-3 py-1.5 text-sm hover:bg-white/5"
          >
            Criar ponto de restauração
          </button>
          <button
            disabled={busy}
            onClick={handleExport}
            className="rounded-md bg-forge-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-forge-accent/80"
          >
            Exportar diagnóstico (JSON)
          </button>
        </div>
      </div>
      {exportMsg && <p className="text-xs text-slate-400">{exportMsg}</p>}
      {restoreMsg && <p className="text-xs text-slate-400">{restoreMsg}</p>}

      <Section title="Trace de DPC/ISR (avançado)">
        <p className="mb-2 text-xs text-slate-500">
          Grava 15s de atividade de kernel com o Windows Performance Recorder (built-in). Se você tiver
          o Windows ADK/xperf instalado, tenta gerar um CSV resumido; caso contrário, só devolve o
          arquivo .etl bruto — abra no{" "}
          <a
            href="https://learn.microsoft.com/windows-hardware/test/wpt/windows-performance-analyzer"
            target="_blank"
            rel="noreferrer"
            className="underline"
          >
            Windows Performance Analyzer
          </a>{" "}
          para ver qual driver está causando latência. Reproduza o problema (jogo travando, áudio
          engasgando) durante os 15s de captura.
        </p>
        <button
          disabled={dpcCapturing}
          onClick={handleCaptureDpcIsr}
          className="rounded-md border border-forge-border px-3 py-1.5 text-sm hover:bg-white/5"
        >
          {dpcCapturing ? "Gravando 15s..." : "Gravar trace de 15s"}
        </button>
        {dpcResult && (
          <div className="mt-2 text-xs text-slate-400">
            {typeof dpcResult === "string" ? (
              <p className="text-forge-danger">Erro: {dpcResult}</p>
            ) : (
              <>
                <p>Trace: {dpcResult.etlPath}</p>
                <p>
                  Resumo CSV:{" "}
                  {dpcResult.csvPath ?? "não disponível (instale o Windows ADK para gerar automaticamente)"}
                </p>
              </>
            )}
          </div>
        )}
      </Section>

      <Section
        title={`Plano de energia atual${powerScheme ? "" : " — não disponível"}`}
        error={powerSchemeError}
      >
        <p className="font-mono text-xs text-slate-400">{powerScheme ?? "—"}</p>
      </Section>

      <Section title="Discos (espaço)" error={disksError}>
        {disks === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : disks.length === 0 ? (
          <p className="text-xs text-slate-500">Nenhum dado disponível.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Unidade</th>
                <th>Rótulo</th>
                <th>Livre</th>
                <th>Total</th>
                <th>Saúde</th>
              </tr>
            </thead>
            <tbody>
              {disks.map((d, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1">{d.driveLetter ?? "—"}:</td>
                  <td>{d.label ?? "—"}</td>
                  <td>{d.freeGb ?? "—"} GB</td>
                  <td>{d.sizeGb ?? "—"} GB</td>
                  <td className={d.healthStatus === "Healthy" ? "text-forge-safe" : "text-forge-warn"}>
                    {d.healthStatus ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="Discos físicos (SMART/saúde)" error={physicalDisksError}>
        {physicalDisks === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : physicalDisks.length === 0 ? (
          <p className="text-xs text-slate-500">Nenhum dado disponível.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Nome</th>
                <th>Tipo</th>
                <th>Saúde</th>
                <th>Status operacional</th>
              </tr>
            </thead>
            <tbody>
              {physicalDisks.map((d, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1">{d.friendlyName}</td>
                  <td>{d.mediaType}</td>
                  <td className={d.healthStatus === "Healthy" ? "text-forge-safe" : "text-forge-danger"}>
                    {d.healthStatus}
                  </td>
                  <td>{d.operationalStatus}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="GPU" error={gpusError}>
        {gpus === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Nome</th>
                <th>Driver</th>
                <th>VRAM</th>
              </tr>
            </thead>
            <tbody>
              {gpus.map((g, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1">{g.name}</td>
                  <td>{g.driverVersion ?? "—"}</td>
                  <td>{g.adapterRam ? `${(g.adapterRam / 1024 / 1024).toFixed(0)} MB` : "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="Aplicativos de inicialização (chave Run)" error={startupAppsError}>
        {startupApps === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : startupApps.length === 0 ? (
          <p className="text-xs text-slate-500">Nenhum item encontrado.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Nome</th>
                <th>Comando</th>
                <th>Local</th>
              </tr>
            </thead>
            <tbody>
              {startupApps.map((a, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1">{a.name}</td>
                  <td className="max-w-md truncate font-mono text-xs text-slate-400">{a.command}</td>
                  <td>{a.location}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="Serviços do Windows" error={servicesError}>
        {services === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Nome</th>
                <th>Exibição</th>
                <th>Status</th>
                <th>Inicialização</th>
              </tr>
            </thead>
            <tbody>
              {services.map((s, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1 font-mono text-xs">{s.name}</td>
                  <td>{s.displayName}</td>
                  <td className={s.status === "Running" ? "text-forge-safe" : "text-slate-400"}>{s.status}</td>
                  <td>{s.startType}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      <Section title="Drivers de dispositivo" error={driversError}>
        <p className="mb-2 text-xs text-slate-500">
          Data do driver é só um sinal, não indica defeito por si só.
        </p>
        {drivers === null ? (
          <p className="text-xs text-slate-500">Carregando...</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-slate-400">
                <th>Dispositivo</th>
                <th>Versão</th>
                <th>Data</th>
                <th>Assinado</th>
              </tr>
            </thead>
            <tbody>
              {drivers.map((d, i) => (
                <tr key={i} className="border-t border-forge-border/50">
                  <td className="py-1">{d.deviceName ?? "—"}</td>
                  <td>{d.driverVersion ?? "—"}</td>
                  <td>{d.driverDate ?? "—"}</td>
                  <td className={d.isSigned ? "text-forge-safe" : "text-forge-warn"}>
                    {d.isSigned === null ? "—" : d.isSigned ? "Sim" : "Não"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>
    </div>
  );
}
