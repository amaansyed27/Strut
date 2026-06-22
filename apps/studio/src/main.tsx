import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./preview3d.css";
import "./engine-cleanup.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
