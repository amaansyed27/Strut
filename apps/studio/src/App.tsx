import { useEffect, useMemo, useState } from "react";
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
  sample_name: string;
  sample_source: string;
  sample_artboards: number;
  sample_state_machines: number;
};

type StrutNode = {
  id: string;
  name: string;
  kind: string;
};

type Artboard = {
  id: string;
  name: string;
  width: number;
  height: number;
  nodes: StrutNode[];
};

type Timeline = {
  id: string;
  name: string;
  duration_ms: number;
};

type StateMachine = {
  id: string;
  name: string;
  states: string[];
};

type StrutDocument = {
  id: string;
  name: string;
  artboards: Artboard[];
  timelines: Timeline[];
  state_machines: StateMachine[];
  bindings: Array<{ name: string }>;
  events: Array<{ name: string }>;
};

const fallbackDocument: StrutDocument = {
  id: "00000000-0000-0000-0000-000000000100",
  name: "Minimal Bot",
  artboards: [
    {
      id: "00000000-0000-0000-0000-000000000101",
      name: "MinimalBot",
      width: 960,
      height: 540,
      nodes: [
        { id: "102", name: "BotRig", kind: "group" },
        { id: "103", name: "GroundShadow", kind: "ellipse" },
        { id: "104", name: "HelmetShell", kind: "path" },
        { id: "105", name: "FacePanel", kind: "rect" },
        { id: "106", name: "Eyes", kind: "path" },
        { id: "107", name: "Smile", kind: "path" },
        { id: "108", name: "Torso", kind: "path" },
        { id: "109", name: "ChestLight", kind: "ellipse" },
        { id: "110", name: "LeftArm", kind: "path" },
        { id: "111", name: "RightArm", kind: "path" },
        { id: "112", name: "LeftLeg", kind: "path" },
        { id: "113", name: "RightLeg", kind: "path" },
        { id: "114", name: "Antennae", kind: "path" },
      ],
    },
  ],
  timelines: [
    { id: "120", name: "idle_float", duration_ms: 1400 },
    { id: "121", name: "wave", duration_ms: 900 },
    { id: "122", name: "blink", duration_ms: 420 },
    { id: "123", name: "scan", duration_ms: 1200 },
    { id: "124", name: "celebrate", duration_ms: 1000 },
  ],
  state_machines: [
    {
      id: "130",
      name: "BotMoods",
      states: ["idle", "float", "wave", "blink", "scan", "celebrate", "sleep"],
    },
  ],
  bindings: [{ name: "face_glow" }, { name: "body_tint" }],
  events: [{ name: "wave_started" }, { name: "celebration_complete" }],
};

const layerColors: Record<string, string> = {
  group: "#2dffb8",
  rect: "#8be9fd",
  ellipse: "#f6d365",
  path: "#ff6b35",
  text: "#f6f1e8",
  image: "#9ba3b4",
  hit_area: "#c7a7ff",
};

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function BotPreview({ activeState }: { activeState: string }) {
  return (
    <svg
      className={`bot-preview state-${activeState}`}
      data-state={activeState}
      data-testid="bot-preview"
      viewBox="0 0 960 540"
      role="img"
      aria-label={`Minimal bot preview in ${activeState} state`}
    >
      <rect width="960" height="540" rx="0" className="bot-sky" />
      <g className="confetti" aria-hidden="true">
        <circle cx="318" cy="118" r="8" />
        <rect x="598" y="102" width="12" height="12" rx="3" />
        <circle cx="644" cy="172" r="7" />
        <rect x="374" y="92" width="10" height="10" rx="2" />
      </g>
      <ellipse className="bot-shadow" cx="480" cy="442" rx="116" ry="18" />
      <g className="bot-rig">
        <g className="antennae">
          <path d="M376 172 L346 112" />
          <circle cx="342" cy="104" r="9" />
          <path d="M584 166 L616 86" />
          <circle cx="620" cy="78" r="9" />
        </g>

        <g className="left-arm">
          <path d="M332 304 C268 320 252 372 282 398 C310 424 352 388 382 344" />
          <circle cx="278" cy="400" r="13" />
          <circle cx="298" cy="414" r="10" />
        </g>
        <g className="right-arm">
          <path d="M624 286 C686 296 716 252 690 218 C666 186 622 214 596 260" />
          <circle cx="696" cy="216" r="12" />
          <circle cx="716" cy="228" r="10" />
          <circle cx="706" cy="246" r="9" />
        </g>

        <g className="legs">
          <path d="M420 376 C390 410 392 448 426 454 C458 460 476 424 482 388" />
          <path d="M532 382 C552 424 584 448 612 424 C636 402 610 366 570 344" />
        </g>

        <g className="body">
          <path d="M372 274 C386 226 430 204 492 206 C558 208 602 236 612 288 C624 356 576 402 486 400 C398 398 350 346 372 274Z" />
          <circle className="chest-light" cx="512" cy="308" r="17" />
          <path className="body-seam" d="M430 358 C466 374 526 374 562 354" />
        </g>

        <g className="helmet">
          <path className="helmet-shell" d="M330 154 C348 70 434 38 544 58 C628 74 662 142 642 224 C620 312 540 350 432 328 C354 312 312 236 330 154Z" />
          <path className="helmet-rim" d="M364 150 C390 96 452 72 530 84 C588 94 618 132 614 188 C610 252 558 286 462 278 C394 272 344 216 364 150Z" />
          <rect className="face-panel" x="386" y="118" width="214" height="136" rx="42" />
          <rect className="scan-line" x="404" y="136" width="178" height="12" rx="6" />
          <path className="eye left-eye" d="M430 176 C438 154 462 154 470 176" />
          <path className="eye right-eye" d="M518 174 C526 152 550 152 558 174" />
          <path className="smile" d="M464 210 C480 230 512 230 528 208" />
          <g className="ears">
            <path d="M330 170 C300 180 290 232 316 256 C340 278 354 238 348 196" />
            <path d="M646 166 C678 180 684 232 656 254 C632 274 620 236 626 194" />
          </g>
        </g>
      </g>
      <text className="bot-state-label" x="480" y="506" textAnchor="middle">
        {titleCase(activeState)}
      </text>
    </svg>
  );
}

function App() {
  const [status, setStatus] = useState<StudioStatus | null>(null);
  const [document, setDocument] = useState<StrutDocument>(fallbackDocument);
  const [activeState, setActiveState] = useState("wave");

  useEffect(() => {
    invoke<StudioStatus>("studio_status").then(setStatus).catch(() => {
      setStatus(null);
    });

    invoke<StrutDocument>("sample_document")
      .then((loadedDocument) => {
        setDocument(loadedDocument);
        const firstState = loadedDocument.state_machines[0]?.states[0];
        if (firstState) {
          setActiveState(firstState);
        }
      })
      .catch(() => {
        setDocument(fallbackDocument);
      });
  }, []);

  const activeArtboard = document.artboards[0] ?? fallbackDocument.artboards[0];
  const activeMachine = document.state_machines[0] ?? fallbackDocument.state_machines[0];
  const states = activeMachine.states;
  const totalTimelineMs = useMemo(
    () => document.timelines.reduce((total, timeline) => total + timeline.duration_ms, 0),
    [document.timelines],
  );

  return (
    <main className="studio-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <img src="/strut-mark.svg" alt="" />
          <div>
            <strong>Strut Studio</strong>
            <span>{status?.format_version ?? "format 0.1.0"} - desktop alpha</span>
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
            <small>{activeArtboard.nodes.length}</small>
          </div>
          <div className="layer-list">
            {activeArtboard.nodes.map((layer) => (
              <button className="layer-row" key={layer.id} type="button">
                <i style={{ background: layerColors[layer.kind] ?? "#9ba3b4" }} />
                <span>{layer.name}</span>
                <em>{layer.kind}</em>
              </button>
            ))}
          </div>

          <div className="panel-heading state-heading">
            <span>
              <Route size={16} />
              {activeMachine.name}
            </span>
          </div>
          <div className="state-grid">
            {states.map((state) => (
              <button
                className={state === activeState ? "active" : ""}
                data-state-button={state}
                key={state}
                type="button"
                onClick={() => setActiveState(state)}
              >
                {titleCase(state)}
              </button>
            ))}
          </div>
        </aside>

        <section className="stage-column">
          <div className="stage-toolbar">
            <span title={status?.sample_source ?? "browser fallback sample"}>
              {document.name}.strut - {activeArtboard.name}
            </span>
            <div>
              <button type="button">100%</button>
              <button type="button">Fit</button>
              <button type="button">Grid</button>
            </div>
          </div>

          <div className="stage">
            <div className="artboard bot-artboard">
              <BotPreview activeState={activeState} />
            </div>
          </div>

          <footer className="timeline-panel">
            <div className="timeline-header">
              <span>Timeline</span>
              <small>{totalTimelineMs}ms total</small>
            </div>
            <div className="timeline-ruler">
              {document.timelines.map((timeline, index) => {
                const start = 2 + index * 18;
                const width = Math.max(10, Math.min(20, timeline.duration_ms / 70));
                const isActive =
                  activeState === timeline.name ||
                  (activeState === "float" && timeline.name === "idle_float");

                return (
                  <div
                    className={`timeline-clip ${isActive ? "active" : ""}`}
                    key={timeline.id}
                    style={{
                      left: `${start}%`,
                      width: `${width}%`,
                      borderColor: isActive ? "#2dffb8" : "#8be9fd",
                    }}
                  >
                    {titleCase(timeline.name)}
                  </div>
                );
              })}
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
              <span>{document.timelines.length} timelines loaded</span>
              <strong>ready</strong>
            </div>
            <div>
              <ScanSearch size={16} />
              <span>{states.length} states reachable</span>
              <strong>ready</strong>
            </div>
            <div>
              <Braces size={16} />
              <span>{document.bindings.length} runtime bindings</span>
              <strong>ready</strong>
            </div>
          </div>
        </aside>
      </section>
    </main>
  );
}

export default App;
