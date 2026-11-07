// Loads the built stargazer.wasm the way the web page's worker does and
// plays a few moves through it: proves the exports work from JavaScript.
// Usage: node web/tests/smoke.mjs path/to/stargazer.wasm
import { readFileSync } from "node:fs";

const bytes = readFileSync(process.argv[2]);
const { instance } = await WebAssembly.instantiate(bytes, {
  env: { now: () => performance.now() },
});
const engine = instance.exports;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function call(name, text = "", ...args) {
  const data = encoder.encode(text);
  new Uint8Array(engine.memory.buffer, engine.input(data.length), data.length).set(data);
  const length = engine[name](...args);
  return decoder.decode(new Uint8Array(engine.memory.buffer, engine.output(), length));
}

function check(ok, message) {
  if (!ok) {
    console.error("FAILED:", message);
    process.exit(1);
  }
}

const version = call("version");
check(/^Stargazer \d+\.\d+\.\d+$/.test(version), `version: ${version}`);
let state = JSON.parse(call("new_game", ""));
check(state.legal.length === 20 && state.status === "playing", "a new game");
const played = JSON.parse(call("play", "e2e4"));
check(played.san === "e4" && played.turn === "b", "playing e2e4");
check(JSON.parse(call("play", "e2e4")).error, "an illegal move is refused");
const started = performance.now();
const thought = JSON.parse(call("think", "", 300));
const took = performance.now() - started;
check(played.legal.includes(thought.move), `the engine's move ${thought.move} is legal`);
check(took < 1000, `thought for ${took} ms`);
const reply = JSON.parse(call("play", thought.move));
check(reply.turn === "w", "the engine's move is played");
console.log(`ok: ${version}, replied ${reply.san} (depth ${thought.depth}, ${thought.score}) in ${Math.round(took)} ms`);
