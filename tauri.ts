import { invoke } from "@tauri-apps/api/tauri";
import type {
  Profile,
  Optimization,
  ExecutionSession,
  ExecutionLogEntry,
  ProcessInfo,
  DetectedGame,
  TelemetryComparison,
  CleanupReport,
  StartupAppDiag,
  ServiceDiag,
  DiskDiag,
  PhysicalDiskDiag,
  GpuDiag,
  DriverDiag,
} from "@/types";

/**
 * Camada única de acesso ao backend Rust. A UI nunca chama `invoke` diretamente
 * fora deste arquivo — assim todo comando exposto pelo Core fica auditável em um lugar.
 */
export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  applyProfile: (profileId: string) => invoke<ExecutionSession>("apply_profile", { profileId }),
  restoreProfile: (profileId: string) => invoke<void>("restore_profile", { profileId }),
  restoreSession: (sessionId: string) => invoke<void>("restore_session", { sessionId }),

  listCatalog: () => invoke<Optimization[]>("list_optimizations"),
  applyOptimization: (optimizationId: string) =>
    invoke<void>("apply_optimization", { optimizationId }),
  restoreOptimization: (optimizationId: string) =>
    invoke<void>("restore_optimization", { optimizationId }),

  listSessions: () => invoke<ExecutionSession[]>("list_sessions"),
  listLogs: (sessionId?: string) => invoke<ExecutionLogEntry[]>("list_logs", { sessionId }),

  systemStatus: () =>
    invoke<{
      cpuUsage: number;
      ramUsage: number;
      windowsBuild: string;
      isAdmin: boolean;
    }>("get_system_status"),
  listTopProcesses: (count?: number) => invoke<ProcessInfo[]>("list_top_processes", { count }),

  listDetectedGames: () => invoke<DetectedGame[]>("list_detected_games"),
  registerGame: (params: {
    nome: string;
    executableName: string;
    launcher?: string;
    profileId?: string;
  }) => invoke<string>("register_game", params),
  setGameProfile: (gameId: string, profileId: string | null) =>
    invoke<void>("set_game_profile", { gameId, profileId }),
  deleteGame: (gameId: string) => invoke<void>("delete_game", { gameId }),

  listTelemetryComparisons: () => invoke<TelemetryComparison[]>("list_telemetry_comparisons"),
  getTelemetryOptIn: () => invoke<boolean>("get_telemetry_opt_in"),
  setTelemetryOptIn: (enabled: boolean) => invoke<void>("set_telemetry_opt_in", { enabled }),

  cleanTempFiles: () => invoke<CleanupReport>("clean_temp_files"),

  createRestorePoint: (description: string) => invoke<void>("create_restore_point", { description }),
  listStartupApps: () => invoke<StartupAppDiag[]>("list_startup_apps"),
  listServicesDiagnostic: () => invoke<ServiceDiag[]>("list_services_diagnostic"),
  listDisks: () => invoke<DiskDiag[]>("list_disks"),
  listPhysicalDisks: () => invoke<PhysicalDiskDiag[]>("list_physical_disks"),
  listGpus: () => invoke<GpuDiag[]>("list_gpus"),
  listDrivers: () => invoke<DriverDiag[]>("list_drivers"),
  getCurrentPowerScheme: () => invoke<string | null>("get_current_power_scheme"),
  exportDiagnostics: () => invoke<string>("export_diagnostics"),

  captureDpcIsr: (durationSecs: number) =>
    invoke<{ etlPath: string; csvPath: string | null }>("capture_dpc_isr", { durationSecs }),
};
