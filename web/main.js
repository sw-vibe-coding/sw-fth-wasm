import init, { Machine } from "./pkg/proto_forth_wasm.js";

let machine = null;

function render() {
  document.getElementById("stackPane").value = machine.get_stack_text();
  document.getElementById("dictPane").value = machine.get_dictionary_text();
  document.getElementById("outputPane").value = machine.get_output_text();
  document.getElementById("historyPane").value = machine.get_history_text();
  document.getElementById("tracePane").value = machine.get_trace_text();
}

async function main() {
  await init();
  machine = new Machine();
  render();

  const repl = document.getElementById("repl");
  const sourcePane = document.getElementById("sourcePane");

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
