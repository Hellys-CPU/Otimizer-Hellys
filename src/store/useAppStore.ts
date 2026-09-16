import { create } from "zustand";
import type { Profile, Optimization, ExecutionSession } from "@/types";
import { api } from "@/lib/tauri";

interface AppState {
  profiles: Profile[];
  catalog: Optimization[];
  activeSession: ExecutionSession | null;
  loading: boolean;
  error: string | null;

  loadProfiles: () => Promise<void>;
  loadCatalog: () => Promise<void>;
  applyProfile: (profileId: string) => Promise<void>;
  restoreProfile: (profileId: string) => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  profiles: [],
  catalog: [],
  activeSession: null,
  loading: false,
  error: null,

  loadProfiles: async () => {
    set({ loading: true, error: null });
    try {
      const profiles = await api.listProfiles();
      set({ profiles, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  loadCatalog: async () => {
    set({ loading: true, error: null });
    try {
      const catalog = await api.listCatalog();
      set({ catalog, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  applyProfile: async (profileId: string) => {
    set({ loading: true, error: null });
    try {
      const session = await api.applyProfile(profileId);
      set({ activeSession: session, loading: false });
      await get().loadProfiles();
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  restoreProfile: async (profileId: string) => {
    set({ loading: true, error: null });
    try {
      await api.restoreProfile(profileId);
      set({ loading: false });
      await get().loadProfiles();
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },
}));
