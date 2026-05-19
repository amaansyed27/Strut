import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Bot,
  Braces,
  Circle,
  Cpu,
  Gauge,
  Layers3,
  MousePointer2,
  Pencil,
  Play,
  Route,
  Save,
  ScanSearch,
  Sparkles,
  Square,
  Upload,
  WandSparkles,
  Zap,
} from "lucide-react";
import "./App.css";

type StudioStatus = {
  app: string;
  core_version: string;
  format_version: string;
  sample_artboards: number;
  sample_state_machines: number;
};

const layers = [
  { name: "LoginButton", kind: "component", color: "#2dffb8" },
  { name: "ButtonSurface", kind: "rect", color: "#f6f1e8" },
  { name: "Label", kind: "text", color: "#8be9fd" },
  { name: "SpinnerArc", kind: "path", color: "#f6d365" },
  { name: "SuccessCheck", kind: "path", color: "#ff6b35" },
];

const states = ["idle", "hover", "pressed", "loading", "success", "error"];

const timeline = [
  { label: "idle", start: 2, width: 16, color: "#8be9fd" },
  { label: "hover", start: 22, width: 18, color: "#2dffb8" },
  { label: "press", start: 44, width: 13, color: "#ff6b35" },
  { label: "load", start: 61, width: 23, color: "#f6d365" },
];

function App() {
  const [status, setStatus] = useState<StudioStatus | null>(null);
  const [activeState, setActiveState] = useState("loading");

  useEffect(() => {
    invoke<StudioStatus>("studio_status").then(setStatus).catch(() => {
      setStatus(null);
    });
  }, []);

  return (
    <main className="studio-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <img src="/strut-mark.svg" alt="" />
          <div>
            <strong>Strut Studio</strong>
            <span>{status?.format_version ?? "format 0.1.0"} · desktop alpha</span>
          </div>
        </div>

        <nav className="mode-tabs" aria-label="Studio modes">
          <button className="active" type="button">
            <MousePointer2 size={16} />
            Design
          </button>
          <button type="button">
            <Route size={16} />
            States
          </button>
          <button type="button">
            <Bot size={16} />
            Agent
          </button>
        </nav>

        <div className="topbar-actions">
          <button className="icon-button" type="button" title="Import mockup">
            <Upload size={17} />
          </button>
          <button className="icon-button" type="button" title="Save project">
            <Save size={17} />
          </button>
          <button className="primary-action" type="button">
            <Play size={17} />
            Preview
          </button>
        </div>
      </header>

      <section className="workspace">
        <aside className="tool-rail" aria-label="Tools">
          <button className="selected" type="button" title="Select">
            <MousePointer2 size={20} />
          </button>
          <button type="button" title="Draw path">
            <Pencil size={20} />
          </button>
          <button type="button" title="Rectangle">
            <Square size={20} />
          </button>
          <button type="button" title="Ellipse">
            <Circle size={20} />
          </button>
          <button type="button" title="AI create">
            <WandSparkles size={20} />
          </button>
        </aside>

        <aside className="panel layers-panel">
          <div className="panel-heading">
            <span>
              <Layers3 size={16} />
              Layers
            </span>
            <small>{layers.length}</small>
          </div>
          <div className="layer-list">
            {layers.map((layer) => (
              <button className="layer-row" key={layer.name} type="button">
                <i style={{ background: layer.color }} />
                <span>{layer.name}</span>
                <em>{layer.kind}</em>
              </button>
            ))}
          </div>

          <div className="panel-heading state-heading">
            <span>
              <Route size={16} />
              State Machine
            </span>
          </div>
          <div className="state-grid">
            {states.map((state) => (
              <button
                className={state === activeState ? "active" : ""}
                key={state}
                type="button"
                onClick={() => setActiveState(state)}
              >
                {state}
              </button>
            ))}
          </div>
        </aside>

        <section className="stage-column">
          <div className="stage-toolbar">
            <span>login-button.strut</span>
            <div>
              <button type="button">100%</button>
              <button type="button">Fit</button>
              <button type="button">Grid</button>
            </div>
          </div>

          <div className="stage">
            <div className="artboard">
              <div className="motion-path" />
              <button className={`sample-button ${activeState}`} type="button">
                <span className="spinner" />
                <strong>{activeState === "success" ? "Done" : "Sign in"}</strong>
              </button>
              <div className="node-tag tag-left">hover</div>
              <div className="node-tag tag-right">success</div>
            </div>
          </div>

          <footer className="timeline-panel">
            <div className="timeline-header">
              <span>Timeline</span>
              <small>0ms · 240ms · 520ms · 900ms</small>
            </div>
            <div className="timeline-ruler">
              {timeline.map((item) => (
                <div
                  className="timeline-clip"
                  key={item.label}
                  style={{
                    left: `${item.start}%`,
                    width: `${item.width}%`,
                    borderColor: item.color,
                  }}
                >
                  {item.label}
                </div>
              ))}
            </div>
          </footer>
        </section>

        <aside className="panel agent-panel">
          <div className="panel-heading">
            <span>
              <Sparkles size={16} />
              Agent Run
            </span>
            <small>BYOK</small>
          </div>

          <div className="agent-card">
            <div className="agent-card-title">
              <Zap size={17} />
              Plan Mode
            </div>
            <p>Sketch directions before full generation when no mockup is attached.</p>
            <button className="wide-action" type="button">
              Generate Sketches
            </button>
          </div>

          <div className="provider-stack">
            {["Ollama", "OpenAI", "Anthropic", "Gemini", "OpenRouter"].map((provider) => (
              <button key={provider} type="button">
                <Cpu size={15} />
                {provider}
              </button>
            ))}
          </div>

          <div className="verifier-list">
            <div>
              <Gauge size={16} />
              <span>60fps preview budget</span>
              <strong>ready</strong>
            </div>
            <div>
              <ScanSearch size={16} />
              <span>state reachability</span>
              <strong>ready</strong>
            </div>
            <div>
              <Braces size={16} />
              <span>runtime bindings</span>
              <strong>ready</strong>
            </div>
          </div>
        </aside>
      </section>
    </main>
  );
}

export default App;
