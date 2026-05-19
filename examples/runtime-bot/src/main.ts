import {
  type BotState,
  loadStrutUrl,
  mountStrut,
} from "../../../packages/runtime-web/src/index";
import "./styles.css";

const stage = document.querySelector<HTMLElement>("[data-runtime-stage]");
const controls = document.querySelector<HTMLElement>("[data-runtime-controls]");
const status = document.querySelector<HTMLElement>("#status");

if (!stage || !controls || !status) {
  throw new Error("runtime example shell is missing required elements");
}

const strutPackage = await loadStrutUrl("/samples/minimal-bot.strut");
const player = mountStrut(stage, strutPackage.document, { initialState: "idle" });
const states = strutPackage.document.state_machines[0]?.states ?? [];

status.textContent = `${strutPackage.document.name} loaded - ${states.length} states`;

for (const state of states) {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = state
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
  button.addEventListener("click", () => player.setState(state as BotState));
  controls.append(button);
}
