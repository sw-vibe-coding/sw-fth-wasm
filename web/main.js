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

function restoreFromStorage(el, key) {
  try {
    const v = window.localStorage.getItem(key);
    if (v !== null) el.value = v;
  } catch (_) {
    // localStorage unavailable (private browsing, sandboxed iframe) — ignore
  }
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
  restoreFromStorage(sourcePane, STORAGE_KEY_SOURCE);
  restoreFromStorage(repl, STORAGE_KEY_REPL);
  persistOnInput(sourcePane, STORAGE_KEY_SOURCE);
  persistOnInput(repl, STORAGE_KEY_REPL);

  await init();
  machine = new Machine();
  render();

  document.getElementById("runBtn").addEventListener("click", () => {
    machine.eval_repl(repl.value);
    render();
  });

  document.getElementById("resetBtn").addEventListener("click", () => {
    machine.reset();
    render();
  });

  document.getElementById("loadSourceBtn").addEventListener("click", () => {
    machine.load_source(sourcePane.value);
    render();
  });

  repl.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      machine.eval_repl(repl.value);
      render();
    }
  });
}

main();
