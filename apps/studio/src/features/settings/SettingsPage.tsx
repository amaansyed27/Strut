/**
 * Settings page component.
 *
 * Extracted from App.tsx for modularity.
 */

import { Monitor, Moon, Sun } from "lucide-react";
import type { ThemeMode } from "../../types";

type SettingsPageProps = {
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
};

const themeOptions: Array<{ id: ThemeMode; icon: typeof Sun; label: string }> = [
  { id: "system", icon: Monitor, label: "System" },
  { id: "light", icon: Sun, label: "Light" },
  { id: "dark", icon: Moon, label: "Dark" },
];

export function SettingsPage({ themeMode, setThemeMode }: SettingsPageProps) {
  return (
    <section className="settings-page page-shell">
      <div className="page-heading">
        <h1>Settings</h1>
        <p>Configure Strut Studio preferences.</p>
      </div>

      <div className="settings-section">
        <h2>Appearance</h2>
        <div className="theme-options">
          {themeOptions.map(({ id, icon: Icon, label }) => (
            <button
              aria-pressed={themeMode === id}
              className={themeMode === id ? "active" : ""}
              key={id}
              type="button"
              onClick={() => setThemeMode(id)}
            >
              <Icon size={15} />
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="settings-section">
        <h2>About</h2>
        <p className="settings-about">
          Strut Studio — AI-native motion design studio for interactive product graphics.
        </p>
        <p className="settings-version">Version 1.0.0</p>
      </div>
    </section>
  );
}
