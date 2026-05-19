import {
  loadStrutUrl,
  mountStrut,
  type MountedStrut,
} from "../../../packages/runtime-web/src/index";
import "./styles.css";

type Level = {
  title: string;
  kicker: string;
  copy: string;
  hint: string;
  board: string[][];
  sequence: string[];
};

type TilePick = {
  row: number;
  col: number;
};

const levels: Level[] = [
  {
    title: "Find the first trail",
    kicker: "Warm up",
    copy: "Tap the glyphs in the same order as the trail above.",
    hint: "Start with A, then B, then C.",
    sequence: ["A", "B", "C"],
    board: [
      ["A", "X", "C"],
      ["D", "B", "E"],
      ["F", "G", "H"],
    ],
  },
  {
    title: "Orbit order",
    kicker: "Pattern",
    copy: "The path is longer now. Ignore the decoy glyphs.",
    hint: "Read ORBIT in order.",
    sequence: ["O", "R", "B", "I", "T"],
    board: [
      ["O", "A", "R", "N"],
      ["M", "B", "K", "I"],
      ["Q", "D", "T", "L"],
      ["S", "E", "P", "C"],
    ],
  },
  {
    title: "Strut lock",
    kicker: "Memory",
    copy: "Complete the product mark without hitting the wrong tile.",
    hint: "The trail spells STRUT.",
    sequence: ["S", "T", "R", "U", "T"],
    board: [
      ["S", "L", "A", "T"],
      ["P", "R", "N", "O"],
      ["U", "C", "T", "E"],
      ["I", "M", "B", "V"],
    ],
  },
  {
    title: "Signal bridge",
    kicker: "Focus",
    copy: "The mascot scans when you miss. Use the target row to recover.",
    hint: "Follow CODEX from left to right in the target strip.",
    sequence: ["C", "O", "D", "E", "X"],
    board: [
      ["C", "R", "V", "O"],
      ["L", "D", "A", "M"],
      ["N", "E", "P", "X"],
      ["S", "I", "T", "B"],
    ],
  },
  {
    title: "Mascot run",
    kicker: "Final",
    copy: "Clear the last sequence and the Strut mascot crosses the whole screen.",
    hint: "The final answer is MASCOT.",
    sequence: ["M", "A", "S", "C", "O", "T"],
    board: [
      ["M", "Q", "A", "R"],
      ["L", "S", "N", "C"],
      ["O", "D", "T", "E"],
      ["F", "H", "P", "B"],
    ],
  },
];

const boardElement = requiredElement<HTMLElement>("[data-board]");
const sequenceElement = requiredElement<HTMLElement>("[data-sequence]");
const levelCountElement = requiredElement<HTMLElement>("[data-level-count]");
const levelTitleElement = requiredElement<HTMLElement>("[data-level-title]");
const levelKickerElement = requiredElement<HTMLElement>("[data-level-kicker]");
const levelCopyElement = requiredElement<HTMLElement>("[data-level-copy]");
const resetButton = requiredElement<HTMLButtonElement>("[data-reset]");
const nextButton = requiredElement<HTMLButtonElement>("[data-next]");
const clearedElement = requiredElement<HTMLElement>("[data-cleared]");
const streakElement = requiredElement<HTMLElement>("[data-streak]");
const mistakesElement = requiredElement<HTMLElement>("[data-mistakes]");
const hintElement = requiredElement<HTMLElement>("[data-hint]");
const speechElement = requiredElement<HTMLElement>("[data-speech]");
const mascotStatusElement = requiredElement<HTMLElement>("[data-mascot-status]");
const mascotNameElement = requiredElement<HTMLElement>("[data-mascot-name]");
const mascotStage = requiredElement<HTMLElement>("[data-mascot-stage]");
const miniMascotStage = requiredElement<HTMLElement>("[data-mini-mascot]");
const screenMascot = requiredElement<HTMLElement>("[data-screen-mascot]");

let activeLevel = 0;
let activeStep = 0;
let cleared = 0;
let streak = 0;
let mistakes = 0;
let solved = false;
let picks: TilePick[] = [];
let mascot: MountedStrut | null = null;
let miniMascot: MountedStrut | null = null;
let cheerTimer = 0;

const strutPackage = await loadStrutUrl("/samples/game-mascot.strut");
mascot = mountStrut(mascotStage, strutPackage.document, { initialState: "float" });
miniMascot = mountStrut(miniMascotStage, strutPackage.document, { initialState: "blink", reducedMotion: true });
mascot.svg.querySelector("[data-state-label]")?.remove();
miniMascot.svg.querySelector("[data-state-label]")?.remove();
mascotNameElement.textContent = strutPackage.document.name;

resetButton.addEventListener("click", () => restartLevel());
nextButton.addEventListener("click", () => advanceLevel());

renderLevel();
setMascotMood("float", "I will cheer when you solve a trail.", "Watching the trail");

function renderLevel() {
  const level = levels[activeLevel];
  solved = false;
  activeStep = 0;
  picks = [];
  levelCountElement.textContent = `Level ${activeLevel + 1} / ${levels.length}`;
  levelTitleElement.textContent = level.title;
  levelKickerElement.textContent = level.kicker;
  levelCopyElement.textContent = level.copy;
  hintElement.textContent = level.hint;
  nextButton.disabled = true;
  nextButton.textContent = activeLevel === levels.length - 1 ? "Finish run" : "Next level";
  renderSequence(level);
  renderBoard(level);
  renderStats();
}

function renderSequence(level: Level) {
  sequenceElement.replaceChildren();
  for (const [index, glyph] of level.sequence.entries()) {
    const item = document.createElement("span");
    item.className = index < activeStep ? "done" : index === activeStep ? "current" : "";
    item.textContent = glyph;
    sequenceElement.append(item);
  }
}

function renderBoard(level: Level) {
  boardElement.replaceChildren();
  boardElement.style.setProperty("--grid-size", String(level.board.length));
  for (const [rowIndex, row] of level.board.entries()) {
    for (const [colIndex, glyph] of row.entries()) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = picked(rowIndex, colIndex) ? "tile picked" : "tile";
      button.dataset.glyph = glyph;
      button.dataset.row = String(rowIndex);
      button.dataset.col = String(colIndex);
      button.textContent = glyph;
      button.addEventListener("click", () => chooseTile(glyph, rowIndex, colIndex));
      boardElement.append(button);
    }
  }
}

function chooseTile(glyph: string, row: number, col: number) {
  if (solved || picked(row, col)) {
    return;
  }

  const level = levels[activeLevel];
  const expected = level.sequence[activeStep];
  if (glyph !== expected) {
    mistakes += 1;
    streak = 0;
    renderStats();
    pulseBoard("wrong");
    setMascotMood("scan", `Not ${glyph}. Scan for ${expected}.`, "Scanning the board");
    return;
  }

  picks.push({ row, col });
  activeStep += 1;
  streak += 1;
  renderSequence(level);
  renderBoard(level);
  renderStats();
  setMascotMood("wave", `${glyph} locked. Keep going.`, "Cheering the streak");

  if (activeStep === level.sequence.length) {
    completeLevel();
  }
}

function completeLevel() {
  solved = true;
  cleared = Math.max(cleared, activeLevel + 1);
  renderStats();
  nextButton.disabled = false;
  pulseBoard("complete");
  runMascotAcrossScreen(activeLevel === levels.length - 1 ? "Final trail cleared." : "Level cleared.");
}

function restartLevel() {
  renderLevel();
  setMascotMood("blink", "Fresh board. Try the trail again.", "Reset the level");
}

function advanceLevel() {
  if (!solved) {
    return;
  }
  if (activeLevel === levels.length - 1) {
    activeLevel = 0;
    cleared = 0;
    streak = 0;
    mistakes = 0;
    renderLevel();
    setMascotMood("celebrate", "Full run reset. Go again.", "Ready for a new run");
    return;
  }
  activeLevel += 1;
  renderLevel();
  setMascotMood("float", "New level. I am with you.", "Watching the trail");
}

function runMascotAcrossScreen(message: string) {
  if (!mascot) {
    return;
  }
  window.clearTimeout(cheerTimer);
  mascot.setState("celebrate");
  speechElement.textContent = `${message} Nice work.`;
  mascotStatusElement.textContent = "Cross-screen celebration";
  screenMascot.classList.remove("run");
  void screenMascot.offsetWidth;
  screenMascot.classList.add("run");
  cheerTimer = window.setTimeout(() => {
    screenMascot.classList.remove("run");
    mascot?.setState("float");
    speechElement.textContent = activeLevel === levels.length - 1 ? "You cleared the whole run." : "Tap Next level.";
    mascotStatusElement.textContent = "Waiting for the next trail";
  }, 1850);
}

function setMascotMood(state: string, speech: string, status: string) {
  mascot?.setState(state);
  speechElement.textContent = speech;
  mascotStatusElement.textContent = status;
}

function pulseBoard(kind: "wrong" | "complete") {
  boardElement.classList.remove("wrong", "complete");
  void boardElement.offsetWidth;
  boardElement.classList.add(kind);
  window.setTimeout(() => boardElement.classList.remove(kind), 700);
}

function renderStats() {
  clearedElement.textContent = String(cleared);
  streakElement.textContent = String(streak);
  mistakesElement.textContent = String(mistakes);
}

function picked(row: number, col: number) {
  return picks.some((pick) => pick.row === row && pick.col === col);
}

function requiredElement<T extends HTMLElement>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) {
    throw new Error(`missing ${selector}`);
  }
  return element;
}
