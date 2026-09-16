import { NavLink, Route, Routes } from "react-router-dom";
import Dashboard from "@/pages/Dashboard";
import Profiles from "@/pages/Profiles";
import Catalog from "@/pages/Catalog";
import Security from "@/pages/Security";
import Backups from "@/pages/Backups";
import Logs from "@/pages/Logs";
import Games from "@/pages/Games";
import Comparison from "@/pages/Comparison";
import Settings from "@/pages/Settings";

const NAV_ITEMS = [
  { to: "/", label: "Dashboard" },
  { to: "/profiles", label: "Perfis" },
  { to: "/catalog", label: "Catálogo" },
  { to: "/games", label: "Jogos" },
  { to: "/comparison", label: "Comparação" },
  { to: "/security", label: "Segurança" },
  { to: "/backups", label: "Backup e Restauração" },
  { to: "/logs", label: "Logs" },
  { to: "/settings", label: "Configurações" },
];

export default function App() {
  return (
    <div className="flex h-screen">
      <aside className="w-56 shrink-0 border-r border-forge-border bg-forge-panel p-4">
        <h1 className="mb-6 text-lg font-semibold text-slate-100">SystemForge</h1>
        <nav className="flex flex-col gap-1">
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                `rounded-md px-3 py-2 text-sm transition-colors ${
                  isActive
                    ? "bg-forge-accent/20 text-forge-accent"
                    : "text-slate-400 hover:bg-white/5 hover:text-slate-100"
                }`
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-y-auto p-6">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/profiles" element={<Profiles />} />
          <Route path="/catalog" element={<Catalog />} />
          <Route path="/games" element={<Games />} />
          <Route path="/comparison" element={<Comparison />} />
          <Route path="/security" element={<Security />} />
          <Route path="/backups" element={<Backups />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
