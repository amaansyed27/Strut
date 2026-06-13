import React, { useEffect, useRef } from "react";
import { createRoot } from "react-dom/client";
import { mountStrut, type MountedStrut } from "@strut/runtime-web";
import { demos, releaseChecks } from "./strutDemos";
import "./styles.css";

function DemoCard({ demo }: { demo: (typeof demos)[number] }) {
  const mountRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!mountRef.current) return undefined;
    const mounted: MountedStrut = mountStrut(mountRef.current, demo.document, { initialState: demo.state });
    let index = 0;
    const timer = window.setInterval(() => {
      index += 1;
      mounted.setState(index % 2 === 0 ? "idle" : demo.state);
    }, demo.intervalMs);
    return () => {
      window.clearInterval(timer);
      mounted.destroy();
    };
  }, [demo]);

  return (
    <article className="demo-card" data-demo-card>
      <div className="demo-stage" ref={mountRef} aria-label={`${demo.title} Strut animation`} />
      <div>
        <span>{demo.kind}</span>
        <h3>{demo.title}</h3>
        <p>{demo.copy}</p>
      </div>
    </article>
  );
}

function App() {
  return (
    <main>
      <header className="site-nav">
        <a className="brand" href="#top" aria-label="Strut home">
          <img src="/strut-mark.svg" alt="" />
          <span>Strut</span>
        </a>
        <nav aria-label="Primary navigation">
          <a href="#examples">Examples</a>
          <a href="#workflow">Workflow</a>
          <a href="#download">Download</a>
          <a href="#roadmap">Roadmap</a>
        </nav>
      </header>

      <section className="hero" id="top">
        <div className="hero-copy">
          <p className="eyebrow">Open-source motion design for agentic software teams</p>
          <h1>Strut</h1>
          <p className="hero-lede">AI-native asset studio for coding agents, animated product moments, editable SVG, and sprite-python motion.</p>
          <div className="hero-actions">
            <a href="#download">Download preview</a>
            <a href="#examples">Watch live examples</a>
          </div>
        </div>
        <div className="hero-board" aria-label="Live Strut animation examples">
          {demos.slice(0, 3).map((demo) => (
            <DemoCard demo={demo} key={demo.title} />
          ))}
        </div>
      </section>

      <section className="feature-band" id="workflow">
        <div>
          <span>Strut Sprite</span>
          <h2>Prompt to semantic parts, timelines, and validated .strut scenes.</h2>
          <p>Simple assets stay vector-clean. Heavier mascots and product motion use sprite-python plans, validated by Rust before they become durable project files.</p>
        </div>
        <div>
          <span>Agentic CLI</span>
          <h2>Inspect, plan, patch, verify, render, and export from a user project.</h2>
          <p>Coding agents can work without hand-driving the app: operation batches remain the source of truth and failures do not mutate files.</p>
        </div>
        <div>
          <span>Open-source runtime</span>
          <h2>Ship readable animation artifacts instead of opaque blobs.</h2>
          <p>Exports include scene JSON plus React/SVG playback code that agents and humans can inspect, diff, patch, and verify.</p>
        </div>
      </section>

      <section className="examples" id="examples">
        <div className="section-heading">
          <p className="eyebrow">Live examples</p>
          <h2>One studio for static and animated assets.</h2>
          <p>Each card below is mounted with the Strut runtime, not a video. The same files can be inspected by the CLI and edited in Studio.</p>
        </div>
        <div className="demo-grid">
          {demos.map((demo) => (
            <DemoCard demo={demo} key={demo.title} />
          ))}
        </div>
      </section>

      <section className="download" id="download">
        <div>
          <p className="eyebrow">Desktop app</p>
          <h2>Local-first Studio, Rust validation, React/Tauri shell.</h2>
          <p>Use local providers, BYOK models, or the built-in Strut Sprite engine. Export scenes for React apps or let a coding agent integrate the result.</p>
        </div>
        <div className="download-actions">
          <a href="https://github.com/strut/strut/releases" aria-label="Download for Windows">Download for Windows</a>
          <a href="https://github.com/strut/strut/releases" aria-label="Download for macOS">Download for macOS</a>
          <a href="https://github.com/strut/strut/releases" aria-label="Download for Linux">Download for Linux</a>
        </div>
      </section>

      <section className="release" id="roadmap" data-proof="release-checklist">
        <div className="section-heading">
          <p className="eyebrow">v1.0 release gate</p>
          <h2>What has to stay true before launch.</h2>
        </div>
        <div className="release-grid">
          {releaseChecks.map((item) => (
            <article key={item.title}>
              <span>{item.status}</span>
              <h3>{item.title}</h3>
              <p>{item.copy}</p>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
