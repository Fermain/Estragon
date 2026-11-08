import init, { layoutDocument as rawLayout } from "./pkg/estragon_wasm.js";

let ready;
/** Load the WASM module once. Call (and await) before `layoutDocument`. */
export function initLayout() {
  ready ??= init();
  return ready;
}

/** @param {import('./index').Document} doc */
export function layoutDocument(doc) {
  return JSON.parse(rawLayout(JSON.stringify(doc)));
}
