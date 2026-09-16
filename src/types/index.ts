export type RiskLevel = "safe" | "advanced" | "experimental";

export type OptimizationCategory =
  | "registry"
  | "service"
  | "scheduled_task"
  | "startup"
  | "power_plan"
  | "network"
  | "security"
  | "gaming"
  | "ui";

export interface RegistryTweak {
  id: string;
  nome: string;
  categoria: OptimizationCategory;
  descricao: string;
  objetivo: string;
  caminho: string;
  nomeDoValor: string;
  tipoDoValor: "REG_DWORD" | "REG_SZ" | "REG_QWORD" | "REG_BINARY" | "REG_MULTI_SZ";
  valorOriginal: string | null;
  valorRecomendado: string;
  valorAtual: string | null;
  risco: RiskLevel;
  requerAdministrador: boolean;
  requerReinicializacao: boolean;
  sistemasCompatíveis: Array<"10" | "11">;
  perfisPermitidos: string[];
  comandoDeAplicacao: string;
  comandoDeReversao: string;
  fonteTecnica: string;
  ativo: boolean;
}

export interface Optimization {
  id: string;
  nome: string;
  categoria: OptimizationCategory;
  descricao: string;
  beneficioEsperado: string;
  risco: RiskLevel;
  requerReinicializacao: boolean;
  requerAdministrador: boolean;
  valorAtual: string | null;
  valorRecomendado: string;
  fonteTecnica: string;
  ativo: boolean;
}

export interface Profile {
  id: string;
  nome: string;
  descricao: string;
  icone: string;
  cor: string;
  nivelAgressividade: 1 | 2 | 3;
  otimizacoesIds: string[];
  otimizacoesProibidasIds: string[];
  requerAdministrador: boolean;
  ultimaAplicacao: string | null;
  ativo: boolean;
}

export interface ExecutionSession {
  id: string;
  profileId: string;
  startedAt: string;
  finishedAt: string | null;
  status: "pending" | "applied" | "partial" | "failed" | "restored";
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpuUsage: number;
  memoryMb: number;
}

export type Launcher = "steam" | "epic" | "xbox" | "battlenet" | "riot" | "custom";

export interface DetectedGame {
  id: string;
  nome: string;
  executableName: string;
  executablePath: string | null;
  launcher: Launcher | null;
  profileId: string | null;
  lastPlayedAt: string | null;
}

export interface TelemetryComparison {
  gameSessionId: string;
  gameName: string;
  startedAt: string;
  endedAt: string | null;
  beforeCpuUsagePercent: number | null;
  afterCpuUsagePercent: number | null;
  beforeRamUsageMb: number | null;
  afterRamUsageMb: number | null;
  beforeFpsAvg: number | null;
  afterFpsAvg: number | null;
  beforeFpsLow1pct: number | null;
  afterFpsLow1pct: number | null;
}

export interface ExecutionLogEntry {
  id: string;
  sessionId: string;
  command: string;
  user: string;
  timestamp: string;
  result: "success" | "error";
  errorMessage: string | null;
  returnCode: number | null;
  requiresReboot: boolean;
}
