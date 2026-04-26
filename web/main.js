import init, { Machine } from "./pkg/proto_forth_wasm.js";

let machine = null;

function render() {
  const panes = [
    ["stackPane", machine.get_stack_text()],
    ["dictPane", machine.get_dictionary_text()],
    ["outputPane", machine.get_output_text()],
    ["historyPane", machine.get_history_text()],
    ["tracePane", machine.get_trace_text()],
    ["memoryPane", machine.get_memory_text()],
  ];
  for (const [id, text] of panes) {
    const el = document.getElementById(id);
    el.value = text;
    el.scrollTop = el.scrollHeight;
  }
}

function substituteBuildInfo() {
  for (const id of ["build-host", "build-sha", "build-timestamp"]) {
    const el = document.getElementById(id);
    if (el && el.textContent.includes("__BUILD_")) {
      el.textContent = "dev";
    }
  }
}

const STORAGE_KEY_SOURCE = "sw-fth-wasm:source";
const STORAGE_KEY_REPL = "sw-fth-wasm:repl";
const STORAGE_KEY_STATE = "sw-fth-wasm:state";

function restoreFromStorage(el, key) {
  try {
    const v = window.localStorage.getItem(key);
    if (v !== null) {
      el.value = v;
      return true;
    }
  } catch (_) {
    // localStorage unavailable (private browsing, sandboxed iframe) — ignore
  }
  return false;
}

async function loadBootstrap() {
  try {
    const resp = await fetch("./forth-bootstrap.fs");
    if (resp.ok) return await resp.text();
  } catch (_) {
    // Fetch failed (e.g., offline) — proceed without bootstrap.
  }
  return "";
}

function saveMachineState() {
  if (!machine) return;
  try {
    const json = machine.save_state();
    if (json) window.localStorage.setItem(STORAGE_KEY_STATE, json);
  } catch (_) {
    // localStorage quota / unavailable — ignore.
  }
}

function loadMachineState() {
  try {
    const json = window.localStorage.getItem(STORAGE_KEY_STATE);
    if (json) return machine.load_state(json);
  } catch (_) {
    // ignore
  }
  return false;
}

function clearMachineState() {
  try {
    window.localStorage.removeItem(STORAGE_KEY_STATE);
  } catch (_) {}
}

function persistOnInput(el, key) {
  el.addEventListener("input", () => {
    try {
      window.localStorage.setItem(key, el.value);
    } catch (_) {
      // ignore (quota exceeded or unavailable)
    }
  });
}

async function main() {
  substituteBuildInfo();

  const repl = document.getElementById("repl");
  const sourcePane = document.getElementById("sourcePane");
  const sourceRestored = restoreFromStorage(sourcePane, STORAGE_KEY_SOURCE);
  restoreFromStorage(repl, STORAGE_KEY_REPL);
  persistOnInput(sourcePane, STORAGE_KEY_SOURCE);
  persistOnInput(repl, STORAGE_KEY_REPL);

  // Fetch bootstrap source in parallel with WASM init
  const [bootstrap] = await Promise.all([loadBootstrap(), init()]);
  machine = new Machine();
  // Restore saved Machine state if present, else load bootstrap fresh.
  const stateRestored = loadMachineState();
  if (!stateRestored && bootstrap) {
    machine.load_source(bootstrap);
    saveMachineState();
  }
  if (!sourceRestored && bootstrap) {
    sourcePane.value = bootstrap;
    try {
      window.localStorage.setItem(STORAGE_KEY_SOURCE, bootstrap);
    } catch (_) {}
  }
  render();

  document.getElementById("runBtn").addEventListener("click", () => {
    machine.eval_repl(repl.value);
    render();
    saveMachineState();
  });

  document.getElementById("resetBtn").addEventListener("click", () => {
    machine.reset();
    render();
    saveMachineState();
  });

  document.getElementById("loadSourceBtn").addEventListener("click", () => {
    machine.load_source(sourcePane.value);
    render();
    saveMachineState();
  });

  const wipeBtn = document.getElementById("wipeBtn");
  if (wipeBtn) {
    wipeBtn.addEventListener("click", () => {
      if (!window.confirm("Clear saved Machine state and reload bootstrap?")) return;
      clearMachineState();
      window.location.reload();
    });
  }

  repl.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      machine.eval_repl(repl.value);
      render();
      saveMachineState();
    }
  });
}

main();
