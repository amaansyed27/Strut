import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./preview3d.css";
import "./engine-cleanup.css";
import { installCssPreviewRuntime } from "./features/preview/cssPreviewRuntime";

installCssPreviewRuntime();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
